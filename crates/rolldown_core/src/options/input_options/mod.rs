use std::{path::PathBuf, pin::Pin, sync::Arc};

use derivative::Derivative;
use futures::{future, Future, FutureExt};

use crate::{UnaryBuildResult, WarningHandler};

mod input_item;
pub use input_item::*;
mod builtins;
pub use builtins::*;

type PinFutureBox<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub type IsExternal =
  Arc<dyn Fn(&str, Option<&str>, bool) -> PinFutureBox<UnaryBuildResult<bool>> + Send + Sync>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BuildInputOptions {
  pub input: Vec<InputItem>,
  pub treeshake: bool,
  pub cwd: PathBuf,
  #[derivative(Debug = "ignore")]
  pub is_external: IsExternal,
  #[derivative(Debug = "ignore")]
  pub on_warn: WarningHandler,
  pub shim_missing_exports: bool,
  pub preserve_symlinks: bool,
  pub builtins: BuiltinsOptions,
}

impl Default for BuildInputOptions {
  fn default() -> Self {
    Self {
      input: Default::default(),
      treeshake: true,
      cwd: std::env::current_dir().unwrap(),
      is_external: Arc::new(|_, _, _| future::ready(Ok(false)).boxed()),
      on_warn: Arc::new(|err| {
        eprintln!("{}", err);
      }),
      shim_missing_exports: false,
      builtins: Default::default(),
      preserve_symlinks: true,
    }
  }
}
