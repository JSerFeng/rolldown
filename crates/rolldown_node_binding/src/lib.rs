#![feature(box_syntax)]
#![feature(iterator_try_collect)]
#![feature(result_option_inspect)]

#[cfg(all(
  not(all(target_os = "linux", target_env = "musl", target_arch = "aarch64")),
  not(debug_assertions)
))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

pub mod bundler;
pub mod js_build_plugin;
pub mod js_callbacks;
pub mod options;
pub mod output_chunk;
pub mod utils;

scoped_tls::scoped_thread_local!(static NAPI_ENV: napi::Env);
