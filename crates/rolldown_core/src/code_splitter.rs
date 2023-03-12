use std::path::Component;

use hashlink::LinkedHashSet;
use itertools::Itertools;
// use petgraph::stable_graph::NodeIndex;
use rolldown_common::{ChunkId, ModuleId};
use rustc_hash::{FxHashMap, FxHashSet};
use sugar_path::{AsPath, SugarPath};
use swc_core::ecma::atoms::JsWord;
use tracing::instrument;

pub fn uri_to_chunk_name(root: &str, uri: &str) -> String {
  let path = uri.as_path();
  let mut relatived = path.relative(root);
  let _ext = relatived
    .extension()
    .and_then(|ext| ext.to_str())
    .unwrap_or("")
    .to_string();
  relatived.set_extension("");
  let name = itertools::Itertools::intersperse(
    relatived
      .components()
      .filter(|com| matches!(com, Component::Normal(_)))
      .filter_map(|seg| seg.as_os_str().to_str()),
    "_",
  )
  .fold(String::new(), |mut acc, seg| {
    acc.push_str(seg);
    acc
  });
  // name.push('_');
  // name.push_str(&ext);
  name
}

use crate::{BuildInputOptions, Chunk, ChunkGraph, Graph, UnaryBuildResult};

pub(crate) struct CodeSplitter<'me> {
  opts: &'me BuildInputOptions,
  graph: &'me Graph,
  chunk_by_id: FxHashMap<ChunkId, Chunk>,
  entries: Vec<ModuleId>,
  split_point_module_to_chunk: FxHashMap<ModuleId, ChunkId>,
  // chunk_relation_graph: ChunkRelationGraph,
  mod_to_chunks: FxHashMap<ModuleId, FxHashSet<ChunkId>>,
  // The order is only to make the output stable.
  dynamic_entries: LinkedHashSet<ModuleId>,
}

impl<'me> CodeSplitter<'me> {
  pub(crate) fn new(
    entries: Vec<ModuleId>,
    graph: &'me mut Graph,
    opts: &'me BuildInputOptions,
  ) -> Self {
    Self {
      opts,
      graph,
      chunk_by_id: Default::default(),
      entries,
      split_point_module_to_chunk: Default::default(),
      // chunk_relation_graph: Default::default(),
      mod_to_chunks: graph
        .module_by_id
        .keys()
        .map(|k| (k.clone(), Default::default()))
        .collect(),
      dynamic_entries: graph
        .module_by_id
        .values()
        .flat_map(|m| m.dynamic_dependencies())
        // Ignore external module
        .filter(|m| !m.is_external())
        .cloned()
        .collect::<LinkedHashSet<_>>(),
    }
  }
}

#[derive(Debug, Clone)]
#[allow(unused)]
enum QueueAction {
  Enter,
}

#[derive(Debug, Clone)]
#[allow(unused)]
struct QueueItem {
  pub action: QueueAction,
  pub module_id: JsWord,
  pub chunk_id: JsWord,
}

// #[derive(Debug, Default)]
// struct ChunkRelationGraph {
//   graph: petgraph::graph::DiGraph<JsWord, ()>,
//   chunk_id_to_node_idx: FxHashMap<JsWord, NodeIndex>,
//   node_idx_to_chunk_id: FxHashMap<NodeIndex, NodeIndex>,
// }

// impl ChunkRelationGraph {
//   pub fn add_node(&mut self, chunk_id: JsWord) -> NodeIndex {
//     let node_idx = self.graph.add_node(chunk_id.clone());
//     self.chunk_id_to_node_idx.insert(chunk_id, node_idx);
//     self.node_idx_to_chunk_id.insert(node_idx, node_idx);
//     node_idx
//   }

//   pub fn add_edge(&mut self, from: JsWord, to: JsWord) {
//     let from_node_idx = *self
//       .chunk_id_to_node_idx
//       .entry(from)
//       .or_insert_with_key(|from| self.graph.add_node(from.clone()));
//     let to_node_idx = *self
//       .chunk_id_to_node_idx
//       .entry(to)
//       .or_insert_with_key(|id| self.graph.add_node(id.clone()));
//     self.graph.add_edge(from_node_idx, to_node_idx, ());
//   }

//   pub fn contains_edge(&self, from: &JsWord, to: &JsWord) -> bool {
//     let from_node_idx = self.chunk_id_to_node_idx.get(from).unwrap();
//     let to_node_idx = self.chunk_id_to_node_idx.get(to).unwrap();
//     self.graph.contains_edge(*from_node_idx, *to_node_idx)
//   }
// }

impl<'me> CodeSplitter<'me> {
  pub fn analyze_entries(&mut self, mut entries: Vec<ModuleId>, is_entry_chunk: bool) {
    while let Some(entry) = entries.pop() {
      let _exec_order = self.graph.module_by_id[&entry].exec_order();
      let chunk = Chunk::new(
        uri_to_chunk_name(&self.opts.cwd.to_string_lossy(), entry.as_ref()),
        entry.clone(),
        is_entry_chunk,
      );
      self
        .split_point_module_to_chunk
        .insert(entry.clone(), chunk.id.clone());
      if self.chunk_by_id.contains_key(&chunk.id) {
        tracing::info!("Chunk already exists: {:?}", chunk.id);
        return;
      }
      let chunk = self.chunk_by_id.entry(chunk.id.clone()).or_insert(chunk);
      let mut visited_modules: FxHashSet<ModuleId> = Default::default();
      let mut stack = vec![entry];
      while let Some(module_id) = stack.pop() {
        if visited_modules.contains(&module_id) {
          continue;
        } else {
          visited_modules.insert(module_id.clone());
        }

        chunk.modules.insert(module_id.clone());

        self
          .mod_to_chunks
          .get_mut(&module_id)
          .unwrap()
          .insert(chunk.id.clone());

        let module = self.graph.module_by_id.get(&module_id).unwrap();

        stack.extend(module.dependencies().iter().cloned().rev());
      }
    }
  }

  fn collect_shared_modules(&self) -> Vec<ModuleId> {
    self
      .mod_to_chunks
      .iter()
      .filter(|(_, chunks)| chunks.len() > 1)
      .filter(|(module_id, _)| !module_id.is_external())
      .map(|(module_id, _)| module_id.clone())
      .collect()
  }

  // If a module is a split point, we need to remove it from other chunks
  // to prevent duplicated code.
  fn remove_duplicated_module(&mut self, shared_module_id: &ModuleId) {
    let owner_chunk_id = self
      .split_point_module_to_chunk
      .get(shared_module_id)
      .unwrap();

    let chunks_contains_duplicate_modules = self
      .mod_to_chunks
      .get(shared_module_id)
      .unwrap()
      .iter()
      .filter(|chunk_id| *chunk_id != owner_chunk_id)
      .collect_vec();

    chunks_contains_duplicate_modules
      .into_iter()
      .for_each(|chunk_id| {
        let chunk = self.chunk_by_id.get_mut(chunk_id).unwrap();
        chunk.modules.remove(shared_module_id);
      });

    *self.mod_to_chunks.get_mut(shared_module_id).unwrap() =
      FxHashSet::from_iter([owner_chunk_id.clone()]);
  }

  #[instrument(skip_all)]
  pub(crate) fn split(mut self) -> UnaryBuildResult<ChunkGraph> {
    self.analyze_entries(self.entries.clone(), true);
    self.analyze_entries(
      self.dynamic_entries.clone().into_iter().collect_vec(),
      false,
    );
    self.dynamic_entries.clone().iter().for_each(|entry| {
      self.remove_duplicated_module(entry);
    });

    let mut shared_modules = self.collect_shared_modules();
    while let Some(shared_module_id) = shared_modules.pop() {
      self.analyze_entries(vec![shared_module_id.clone()], false);

      self.remove_duplicated_module(&shared_module_id);

      if shared_modules.is_empty() {
        shared_modules = self.collect_shared_modules()
      }
    }

    Ok(ChunkGraph {
      chunk_by_id: self.chunk_by_id,
      split_point_to_chunk: self.split_point_module_to_chunk,
    })
  }
}
