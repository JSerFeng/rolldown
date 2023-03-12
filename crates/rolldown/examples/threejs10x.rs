use std::path::PathBuf;

use rolldown::{Bundler, InputItem, InputOptions, OutputOptions};
use rolldown_tracing::enable_tracing_on_demand;
use sugar_path::SugarPathBuf;

#[tokio::main]
async fn main() {
  let _guard = enable_tracing_on_demand();
  let project_root = std::env::var_os("CARGO_MANIFEST_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      std::env::current_dir()
        .unwrap()
        .join("./crates/rolldown")
        .into_normalize()
    });

  let mut bundler = Bundler::new(InputOptions {
    input: vec![InputItem {
      name: "threejs-10x".to_string(),
      import: project_root
        .join("../../temp/threejs10x/main.js")
        // .join("../../temp/threejs/src/Three.js")
        .to_string_lossy()
        .to_string(),
    }],
    cwd: project_root.join("../../temp/threejs10x/"),
    // .join("../../temp/threejs"),
    ..Default::default()
  });
  bundler
    .write(OutputOptions {
      ..Default::default()
    })
    .await
    .unwrap();
}
