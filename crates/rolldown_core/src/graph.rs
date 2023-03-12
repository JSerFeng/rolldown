use std::sync::Arc;

use derivative::Derivative;
use itertools::Itertools;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use rolldown_common::{ExportedSpecifier, ImportedSpecifier, ModuleId, Symbol, UnionFind};
use rolldown_resolver::Resolver;
use rolldown_tracing::ContextedTracer;
use rustc_hash::FxHashSet as HashSet;
use rustc_hash::{FxHashMap, FxHashSet};
use sugar_path::AsPath;
use swc_core::common::{Mark, SyntaxContext, GLOBALS};
use swc_core::ecma::atoms::{js_word, JsWord};
use tracing::instrument;

use crate::module_loader::ModuleLoader;
use crate::{
  norm_or_ext::NormOrExt, normal_module::NormalModule, ModuleById, UnaryBuildResult, SWC_GLOBALS,
};
use crate::{BuildError, BuildResult, SharedBuildInputOptions, SharedBuildPluginDriver};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Graph {
  pub input_options: SharedBuildInputOptions,
  pub entries: Vec<ModuleId>,
  pub(crate) module_by_id: ModuleById,
  pub(crate) unresolved_mark: Mark,
  pub(crate) unresolved_ctxt: SyntaxContext,
  #[derivative(Debug = "ignore")]
  pub(crate) uf: UnionFind<Symbol>,
  pub(crate) build_plugin_driver: SharedBuildPluginDriver,
  pub(crate) used_symbols: HashSet<Symbol>,
}

impl Graph {
  pub(crate) fn new(
    build_plugin_driver: SharedBuildPluginDriver,
    input_options: SharedBuildInputOptions,
  ) -> Self {
    let (unresolved_mark, unresolved_ctxt) = GLOBALS.set(&SWC_GLOBALS, || {
      let mark = Mark::new();
      let ctxt = SyntaxContext::empty().apply_mark(mark);
      (mark, ctxt)
    });

    Self {
      input_options,
      entries: Default::default(),
      module_by_id: Default::default(),
      unresolved_mark,
      unresolved_ctxt,
      uf: Default::default(),
      build_plugin_driver,
      used_symbols: Default::default(),
    }
  }

  fn fetch_module<'m>(module_by_id: &'m ModuleById, id: &ModuleId) -> &'m NormOrExt {
    module_by_id
      .get(id)
      .unwrap_or_else(|| panic!("Failed to fetch module: {id:?}"))
  }

  fn fetch_normal_module<'m>(module_by_id: &'m ModuleById, id: &ModuleId) -> &'m NormalModule {
    Self::fetch_module(module_by_id, id)
      .as_norm()
      .unwrap_or_else(|| panic!("Expected NormalModule, got ExternalModule({id:?})"))
  }

  fn fetch_module_mut<'m>(module_by_id: &'m mut ModuleById, id: &ModuleId) -> &'m mut NormOrExt {
    module_by_id
      .get_mut(id)
      .unwrap_or_else(|| panic!("Failed to fetch module: {id:?}"))
  }

  fn fetch_normal_module_mut<'m>(
    module_by_id: &'m mut ModuleById,
    id: &ModuleId,
  ) -> &'m mut NormalModule {
    Self::fetch_module_mut(module_by_id, id)
      .as_norm_mut()
      .unwrap_or_else(|| panic!("Expected NormalModule, got ExternalModule({id:?})"))
  }

  pub(crate) fn add_module(&mut self, module: NormOrExt) {
    debug_assert!(!self.module_by_id.contains_key(module.id()));
    self.module_by_id.insert(module.id().clone(), module);
  }

  #[tracing::instrument(skip_all)]
  fn sort_modules(&mut self) {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum Action {
      Enter,
      Exit,
    }

    let mut stack = self
      .entries
      .iter()
      .map(|entry| (Action::Enter, entry))
      .rev()
      .collect_vec();
    let mut dynamic_entries = FxHashSet::default();

    let mut entered_ids: HashSet<&ModuleId> = FxHashSet::default();
    entered_ids.shrink_to(self.module_by_id.len());

    let mut next_exec_order = 0;

    while let Some((action, id)) = stack.pop() {
      let module = self.module_by_id.get(id).unwrap();
      match action {
        Action::Enter => {
          if !entered_ids.contains(id) {
            entered_ids.insert(id);
            stack.push((Action::Exit, id));
            stack.extend(
              module
                .dependencies()
                .iter()
                .rev()
                .map(|id| (Action::Enter, id)),
            );
            dynamic_entries.extend(
              module
                .dynamic_dependencies()
                .iter()
                .map(|id| (Action::Enter, id)),
            )
          }
        }
        Action::Exit => {
          let module_p = module as *const NormOrExt as *mut NormOrExt;
          // safety:
          // 1. linking is a process in single thread
          // 2. We won't touch `id` filed which is borrowed in above
          unsafe { (*module_p).set_exec_order(next_exec_order) }
          next_exec_order += 1;
        }
      }
    }

    // start again from modules imported dynamically
    stack.extend(dynamic_entries);

    while let Some((action, id)) = stack.pop() {
      let module = self.module_by_id.get(id).unwrap();
      match action {
        Action::Enter => {
          if !entered_ids.contains(id) {
            entered_ids.insert(id);
            stack.push((Action::Exit, id));
            stack.extend(
              module
                .dependencies()
                .iter()
                .rev()
                .map(|id| (Action::Enter, id)),
            );
          }
        }
        Action::Exit => {
          let module_p = module as *const NormOrExt as *mut NormOrExt;
          // safety:
          // 1. linking is a process in single thread
          // 2. We won't touch `id` filed which is borrowed in above
          unsafe { (*module_p).set_exec_order(next_exec_order) }
          next_exec_order += 1;
        }
      }
    }

    tracing::debug!(
      "sorted modules {:#?}",
      self
        .module_by_id
        .values()
        .sorted_by_key(|m| {
          assert_ne!(m.exec_order(), usize::MAX);
          m.exec_order()
        })
        .map(|m| m.id())
        .collect_vec()
    );
  }

  #[instrument(skip_all)]
  fn link(&mut self) -> UnaryBuildResult<()> {
    let mut order_modules = self
      .module_by_id
      .values()
      .map(|module| module.id().clone())
      .collect::<Vec<_>>();
    order_modules.sort_unstable_by_key(|id| self.module_by_id[id].exec_order());

    self.link_exports(&order_modules)?;
    self.link_imports(&order_modules)?;

    Ok(())
  }

  /// Example
  /// ```ts
  /// // index.ts
  /// export { foo } from "./foo.ts";
  /// export { bar } from "./bar.ts";
  /// ```
  /// If `index.js` is importer, `foo.ts` and `bar.ts` are importee.
  /// `foo` and `bar` are `ReExportedSpecifier`s.
  #[instrument(skip_all)]
  fn link_exports(&mut self, order_modules: &[ModuleId]) -> UnaryBuildResult<()> {
    order_modules
      .iter()
      .filter(|importer_id| {
        // Fast path
        !importer_id.is_external()
      })
      .try_for_each(|importer_id| -> UnaryBuildResult<()> {
        let importee_and_re_exports = Self::fetch_normal_module(&self.module_by_id, importer_id)
          .re_exported_ids
          .iter()
          .map(|(importee_id, re_exported_specifier)| {
            (importee_id.clone(), re_exported_specifier.clone())
          })
          .collect::<Vec<_>>();

        importee_and_re_exports.into_iter().try_for_each(
          |(importee_id, re_exports)| -> UnaryBuildResult<()> {
            if importer_id == &importee_id {
              let importee = Self::fetch_normal_module_mut(&mut self.module_by_id, &importee_id);
              re_exports
                .into_iter()
                .try_for_each(|spec| -> UnaryBuildResult<()> {
                  importee.suggest_name(&spec.imported, &spec.exported_as);
                  if spec.imported == js_word!("*") {
                    importee.mark_namespace_id_referenced();
                  }
                  if let Some(original_spec) = importee.find_exported(&spec.imported) {
                    importee.add_to_linked_exports(spec.exported_as, original_spec.clone());
                    Ok(())
                  } else {
                    // A module try to re-export binding from itself. If the binding
                    // is not exist, we won't do any shimming no matter if `enabling shim_missing_exports` is enable

                    Err(BuildError::circular_reexport(
                      spec.imported.to_string(),
                      importee_id.as_ref().as_path().to_path_buf(),
                    ))
                  }
                })?;

              return Ok(());
            };

            let [importer, importee] = self
              .module_by_id
              .get_many_mut([importer_id, &importee_id])
              .unwrap();

            let importer = importer.expect_norm_mut();
            match importee {
              NormOrExt::Normal(importee) => {
                for spec in re_exports {
                  importee.suggest_name(&spec.imported, &spec.exported_as);
                  // Case: export * as foo from './foo'
                  if spec.imported == js_word!("*") {
                    importee.mark_namespace_id_referenced();
                  }
                  if self.input_options.shim_missing_exports
                    && shim_missing_export_if_needed(importee, &spec.imported)
                  {
                    (self.input_options.on_warn)(BuildError::shimmed_export(
                      spec.imported.to_string(),
                      importee_id.as_path().to_path_buf(),
                    ));
                  }
                  if let Some(original_spec) = importee.find_exported(&spec.imported) {
                    importer.add_to_linked_exports(spec.exported_as, original_spec.clone());
                  } else {
                    return Err(BuildError::missing_export(
                      spec.imported.to_string(),
                      importer_id.as_ref(),
                      importee_id.as_ref(),
                    ));
                  }
                }
              }
              NormOrExt::External(importee) => {
                // We will transform
                // ```js
                // export { resolve } from 'path'
                // ```
                // to
                // ```
                // import { resolve } from 'path'
                // export { resolve }
                // ```
                re_exports.into_iter().for_each(|spec| {
                  let symbol_in_importer =
                    importer.create_top_level_symbol(if spec.exported_as != js_word!("default") {
                      &spec.exported_as
                    } else {
                      &spec.imported
                    });

                  importer.add_to_linked_imports(
                    &importee.id,
                    ImportedSpecifier {
                      imported_as: symbol_in_importer.clone(),
                      imported: spec.imported.clone(),
                    },
                  );

                  importer.add_to_linked_exports(
                    spec.exported_as.clone(),
                    ExportedSpecifier {
                      exported_as: spec.exported_as,
                      local_id: symbol_in_importer,
                      /// NOTE: This is a local export to importer
                      owner: importer_id.clone(),
                    },
                  )
                });
              }
            }
            Ok(())
          },
        )?;

        // Process re-export all

        let importee_of_being_re_exported_all =
          Self::fetch_normal_module_mut(&mut self.module_by_id, importer_id)
            .re_export_all
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let non_conflicted_names = {
          use std::collections::hash_map::Entry;
          let mut tmp: FxHashMap<&JsWord, Option<&ExportedSpecifier>> = FxHashMap::default();
          importee_of_being_re_exported_all
            .iter()
            .filter_map(|importee_id| Self::fetch_module(&self.module_by_id, importee_id).as_norm())
            .flat_map(|each_importee| each_importee.linked_exports.iter())
            .for_each(|(exported_name, spec)| match tmp.entry(exported_name) {
              Entry::Occupied(mut entry) => {
                match entry.get() {
                  Some(existed_spec) => {
                    // The name is not first seen, we need to check if the specifiers are the same
                    if *existed_spec == spec {
                      // The specifiers are the same, so it's ok
                    } else {
                      // Mark the name as conflicted
                      entry.insert(None);
                    }
                  }
                  None => {
                    // Already conflicted, just ignore the name
                  }
                }
              }
              Entry::Vacant(entry) => {
                // The name is first seen, so it's ok
                entry.insert(Some(spec));
              }
            });
          tmp
            .into_iter()
            .filter_map(|(name, spec)| spec.map(|_| name.clone()))
            .collect::<FxHashSet<_>>()
        };

        let importer = Self::fetch_module(&self.module_by_id, importer_id).expect_norm();

        let explicit_exported_names_of_importer = importer
          .linked_exports
          .keys()
          .cloned()
          .collect::<FxHashSet<_>>();

        importee_of_being_re_exported_all
          .iter()
          .for_each(|importee_id| {
            // It seems meaningless to re-export all from itself
            if importee_id == importer_id {
              return;
            }

            let [importer, importee] = self
              .module_by_id
              .get_many_mut([importer_id, importee_id])
              .unwrap();
            let importer = importer.expect_norm_mut();

            if let NormOrExt::Normal(importee) = importee {
              importee.re_export_all.iter().for_each(|id| {
                importer.re_export_all.get_or_insert(id.clone());
              });
            }

            match importee {
              NormOrExt::Normal(importee) => {
                importee
                  .linked_exports
                  .clone()
                  .into_iter()
                  .filter(|(name, _)| {
                    // export * from ... does not re-export `default`
                    let is_default_export = name == "default";
                    !is_default_export
                  })
                  .filter(|(name, _)| {
                    // explicit named export has higher priority than names from re-export-all
                    let is_already_exported_explicitly =
                      explicit_exported_names_of_importer.contains(name);

                    !is_already_exported_explicitly
                  })
                  .filter(|(exported_as, _spec)| {
                    // Conflicted names should be hidden
                    non_conflicted_names.contains(exported_as)
                  })
                  .for_each(|(exported_as, spec)| {
                    importer.add_to_linked_exports(exported_as, spec);
                  });

                // Handle case
                // ```ts
                // // index.ts
                // export * from "./foo.ts";
                // // foo.ts
                // export * from "./bar.ts";
                // export * from "external";
                // ```
                importee.re_export_all.iter().for_each(|id| {
                  importer.re_export_all.get_or_insert(id.clone());
                });
              }
              NormOrExt::External(_importee) => {
                // Handle case
                // ```js
                // // index.js
                // import * as foo from './foo'
                // console.log(foo)
                // // foo.js
                // export * from 'external'
                // ```
                // will be transformed to
                // ```js
                // // foo.js
                // import * as external from 'external'
                // const foo = _mergeNamespace({ __proto__: null}, [external])
                // // index.js
                // console.log(foo)
                // ```
                // TODO: We might need to check if the importer is a already import star from importee
                importer
                  .external_modules_of_re_export_all
                  .get_or_insert(importee_id.clone());
              }
            }
          });
        Ok(())
      })
  }

  /// two things
  /// 1. Union symbol
  /// 2. Generate real ImportedSpecifier for each import and add to `linked_imports`
  #[instrument(skip_all)]
  fn link_imports(&mut self, order_modules: &[ModuleId]) -> UnaryBuildResult<()> {
    order_modules
      .iter()
      .filter(|importer_id| !importer_id.is_external())
      .try_for_each(|importer_id| -> UnaryBuildResult<()> {
        let tracer = ContextedTracer::default()
          .context("link imports".to_string())
          .context(format!("importer: {}", importer_id.as_ref()));
        let importee_and_specifiers = Self::fetch_normal_module(&self.module_by_id, importer_id)
          .imports
          .clone()
          .into_iter()
          .collect::<Vec<_>>();

        importee_and_specifiers.into_iter().try_for_each(
          |(importee_id, specs)| -> UnaryBuildResult<()> {
            let tracer = tracer
              .clone()
              .context(format!("importee: {}", importee_id.as_ref()));
            if importer_id == &importee_id {
              // Handle self import
              let importee = Self::fetch_normal_module_mut(&mut self.module_by_id, &importee_id);
              let mut specs = specs.into_iter().collect_vec();
              specs.sort_unstable_by_key(|spec| spec.imported.clone());
              for imported_spec in specs {
                if &imported_spec.imported == "*" {
                  importee.mark_namespace_id_referenced();
                }
                importee.suggest_name(&imported_spec.imported, imported_spec.imported_as.name());

                if self.input_options.shim_missing_exports
                  && shim_missing_export_if_needed(importee, &imported_spec.imported)
                {
                  (self.input_options.on_warn)(BuildError::shimmed_export(
                    imported_spec.imported.to_string(),
                    importee_id.as_path().to_path_buf(),
                  ));
                }
                if let Some(exported_spec) =
                  importee.find_exported(&imported_spec.imported).cloned()
                {
                  self
                    .uf
                    .union(&imported_spec.imported_as, &exported_spec.local_id);

                  // The importee is also the importer
                  let imported_specifier = ImportedSpecifier {
                    imported_as: imported_spec.imported_as.clone(),
                    imported: exported_spec.exported_as.clone(),
                  };
                  tracer
                    .clone()
                    .context(format!("{:?}", imported_spec))
                    .context(format!("{:?}", exported_spec))
                    .emit_trace(format!(
                      "Add to importee.linked_imports: {:?}",
                      imported_specifier
                    ));
                  importee.add_to_linked_imports(&exported_spec.owner, imported_specifier);
                } else {
                  return Err(BuildError::missing_export(
                    imported_spec.imported.to_string(),
                    importer_id.as_ref(),
                    importee_id.as_ref(),
                  ));
                }
              }
              return Ok(());
            }
            let [importer, importee] = self
              .module_by_id
              .get_many_mut([importer_id, &importee_id])
              .unwrap();
            let importer = importer.expect_norm_mut();

            for imported_spec in specs {
              match importee {
                NormOrExt::Normal(importee) => {
                  if &imported_spec.imported == "*" {
                    importee.mark_namespace_id_referenced();
                  }
                  importee.suggest_name(&imported_spec.imported, imported_spec.imported_as.name());
                  if self.input_options.shim_missing_exports
                    && shim_missing_export_if_needed(importee, &imported_spec.imported)
                  {
                    (self.input_options.on_warn)(BuildError::shimmed_export(
                      imported_spec.imported.to_string(),
                      importee_id.as_path().to_path_buf(),
                    ));
                  }
                  if let Some(exported_spec) =
                    importee.find_exported(&imported_spec.imported).cloned()
                  {
                    self
                      .uf
                      .union(&imported_spec.imported_as, &exported_spec.local_id);

                    let imported_specifier = ImportedSpecifier {
                      imported_as: imported_spec.imported_as.clone(),
                      imported: exported_spec.exported_as.clone(),
                    };
                    tracer
                      .clone()
                      .context(format!("{:?}", imported_spec))
                      .context(format!("{:?}", exported_spec))
                      .emit_trace(format!(
                        "Add to importer.linked_imports: {:#?}",
                        imported_specifier
                      ));
                    importer.add_to_linked_imports(
                      &exported_spec.owner,
                      // Redirect to the owner of the exported symbol
                      imported_specifier,
                    );
                  } else if let Some(first_external_id) = importee
                    .external_modules_of_re_export_all
                    .iter()
                    .next()
                    .cloned()
                  {
                    if importee.external_modules_of_re_export_all.len() > 1 {
                      (self.input_options.on_warn)(BuildError::ambiguous_external_namespaces(
                        imported_spec.imported_as.name().to_string(),
                        importee_id.to_string().into(),
                        first_external_id.to_string().into(),
                        importee
                          .external_modules_of_re_export_all
                          .iter()
                          .map(|id| id.to_string().into())
                          .collect_vec(),
                      ))
                    }

                    let symbol_in_importee =
                      importee.create_top_level_symbol(imported_spec.imported_as.name());

                    importee.add_to_linked_imports(
                      &first_external_id,
                      ImportedSpecifier {
                        imported: imported_spec.imported.clone(),
                        imported_as: symbol_in_importee.clone(),
                      },
                    );

                    importee.add_to_linked_exports(
                      imported_spec.imported.clone(),
                      ExportedSpecifier {
                        exported_as: imported_spec.imported.clone(),
                        local_id: symbol_in_importee.clone(),
                        owner: importee_id.clone(),
                      },
                    );

                    importer.add_to_linked_imports(
                      &importee_id,
                      ImportedSpecifier {
                        imported: imported_spec.imported.clone(),
                        imported_as: imported_spec.imported_as.clone(),
                      },
                    );

                    self
                      .uf
                      .union(&imported_spec.imported_as, &symbol_in_importee);
                  } else {
                    return Err(BuildError::missing_export(
                      imported_spec.imported.to_string(),
                      importer_id.as_ref(),
                      importee_id.as_ref(),
                    ));
                  };
                }
                NormOrExt::External(importee) => {
                  importer.add_to_linked_imports(&importee_id, imported_spec.clone());
                  let exported_symbol_of_importee =
                    importee.find_exported_symbol(&imported_spec.imported);

                  self
                    .uf
                    .union(&imported_spec.imported_as, exported_symbol_of_importee);
                }
              }
            }

            Ok(())
          },
        )
      })
  }

  /// In the function, we will:
  /// 1. TODO: More delicate analysis of import/export star for cross-module namespace export
  /// Only after linking, we can know which imported symbol is "namespace symbol" or declared by user.
  /// 2. Generate actual namespace export AST for each module whose namespace is referenced
  #[instrument(skip_all)]
  fn patch(&mut self) {
    use rayon::prelude::*;
    self
      .module_by_id
      .values_mut()
      .par_bridge()
      .for_each(|module| {
        if let NormOrExt::Normal(module) = module {
          module.generate_namespace_export();
        }
      });
  }

  #[instrument(skip_all)]
  pub(crate) async fn generate_module_graph(&mut self) -> BuildResult<()> {
    let resolver = Arc::new(Resolver::with_cwd(
      self.input_options.cwd.clone(),
      self.input_options.preserve_symlinks,
    ));

    ModuleLoader::new(
      self,
      resolver,
      self.build_plugin_driver.clone(),
      self.input_options.clone(),
    )
    .fetch_all_modules()
    .await?;

    self.sort_modules();
    self.link()?;
    self.patch();
    tracing::trace!("graph after link and patch {:#?}", self);

    if self.input_options.treeshake {
      self.treeshake()?;
    } else {
      self
        .module_by_id
        .values_mut()
        .par_bridge()
        .for_each(|module| {
          match module {
            NormOrExt::Normal(module) => {
              // Because of scope hoisting, we need to remove export/import
              rolldown_swc_visitors::remove_export_and_import(&mut module.ast);
            }
            NormOrExt::External(_ext) => {}
          };
        });
    }
    Ok(())
  }
}

fn shim_missing_export_if_needed(importee: &mut NormalModule, imported_name: &JsWord) -> bool {
  if importee.find_exported(imported_name).is_some() {
    false
  } else {
    importee.shim_missing_export(imported_name);
    true
  }
}
