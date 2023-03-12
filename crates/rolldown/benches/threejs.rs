use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use rolldown::{Bundler, InputItem, InputOptions, OutputOptions};
use sugar_path::SugarPath;

#[cfg(not(target_os = "linux"))]
#[global_allocator]
static GLOBAL: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[tokio::main]
async fn threejs() {
  let project_root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());

  let mut bundler = Bundler::new(InputOptions {
    input: vec![InputItem {
      name: "threejs".to_string(),
      import: project_root
        .join("../../temp/threejs/src/Three.js")
        .normalize()
        .to_string_lossy()
        .to_string(),
    }],
    ..Default::default()
  });
  bundler
    .generate(OutputOptions {
      ..Default::default()
    })
    .await
    .unwrap();
}

#[tokio::main]
async fn threejs10x() {
  let project_root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());

  let mut bundler = Bundler::new(InputOptions {
    input: vec![InputItem {
      name: "threejs-10x".to_string(),
      import: project_root
        .join("../../temp/threejs10x/main.js")
        .to_string_lossy()
        .to_string(),
    }],
    cwd: project_root.join("../../temp/threejs10x/"),
    ..Default::default()
  });
  bundler
    .write(OutputOptions {
      ..Default::default()
    })
    .await
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("generate");

  group
    .sample_size(20)
    .bench_function("threejs", |b| b.iter(threejs))
    .bench_function("threejs10x", |b| b.iter(threejs10x));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
