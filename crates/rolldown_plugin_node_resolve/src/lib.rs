use std::path::{Path, PathBuf};

use nodejs_resolver::{ResolveResult, Resolver};
use rolldown_plugin::{async_trait, BuildPlugin, Context, ResolveArgs, ResolveReturn, ResolvedId};

#[derive(Debug)]
pub struct NodeResolvePlugin {
  resolver: Resolver,
  cwd: PathBuf,
}

pub use nodejs_resolver::Options as ResolverOptions;

impl NodeResolvePlugin {
  pub fn new_boxed(options: ResolverOptions, cwd: PathBuf) -> Box<dyn BuildPlugin> {
    let resolver = Resolver::new(options);
    Box::new(Self { resolver, cwd })
  }
}

#[async_trait::async_trait]
impl BuildPlugin for NodeResolvePlugin {
  fn name(&self) -> rolldown_plugin::PluginName {
    std::borrow::Cow::Borrowed("builtin:node-resolve")
  }

  async fn resolve(&self, _ctx: &mut Context, args: &mut ResolveArgs) -> ResolveReturn {
    let importer = args
      .importer
      .map(|importer| Path::new(importer.as_ref()).parent().unwrap())
      .unwrap_or_else(|| Path::new(&self.cwd));
    let s = self.resolver.resolve(importer, args.specifier);
    if s.is_err() {
      // println!("{args:#?}");
      // println!(
      //   "importer: {:#?}",
      //   &Path::new(importer.as_ref())
      //     .canonicalize()
      //     .unwrap()
      //     .parent()
      //     .unwrap()
      //     .display()
      // );
      return Ok(None);
    }
    let s = s.unwrap();
    match s {
      ResolveResult::Info(info) => Ok(Some(ResolvedId {
        id: info.path().to_string_lossy().to_string(),
        external: false,
      })),
      ResolveResult::Ignored => Ok(None),
    }
  }
}
