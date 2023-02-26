use std::path::PathBuf;

use rolldown_core::{Bundler, InputItem, InputOptions, OutputOptions};

#[tokio::main]
async fn main() {
  let root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let fixture_path = root.join("tests/esbuild/import_star/import_star_unused");
  let dist_dir = root.join("examples/dist");

  let mut bundler = Bundler::new(InputOptions {
    input: vec![InputItem {
      name: "main.js".to_string(),
      import: "./entry".to_string(),
    }],
    cwd: fixture_path,
    ..Default::default()
  });

  let assets = bundler
    .write(OutputOptions {
      dir: Some(dist_dir.to_string_lossy().to_string()),
      ..Default::default()
    })
    .await
    .unwrap();

  println!("assets {assets:#?}")
}
