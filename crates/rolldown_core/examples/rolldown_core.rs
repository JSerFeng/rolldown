use std::path::PathBuf;

use rolldown_core::{Bundler, InputItem, InputOptions, OutputOptions};
use sugar_path::SugarPathBuf;

// The example try to build esm output generated by tsc of `@rolldown/core`.

#[tokio::main]
async fn main() {
  // First, you need to figure out how to build esm output `@rolldown/core`.
  let root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let entry = root.join("../../packages/core/dist/index.js");
  let fixture_path = root.join("../../packages/core/dist/").into_normalize();

  let mut bundler = Bundler::new(InputOptions {
    input: vec![InputItem {
      name: "main.js".to_string(),
      import: entry.to_string_lossy().to_string(),
    }],
    cwd: fixture_path,
    ..Default::default()
  });

  let assets = bundler
    .write(OutputOptions {
      ..Default::default()
    })
    .await
    .unwrap();

  println!("assets {assets:#?}")
}
