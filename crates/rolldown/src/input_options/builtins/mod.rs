use derivative::Derivative;
pub use rolldown_core::TsConfig;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct BuiltinsOptions {
  /// None means disable the builtin
  /// None means default
  pub tsconfig: Option<TsConfig>,
}

impl Default for BuiltinsOptions {
  fn default() -> Self {
    Self {
      tsconfig: Some(Default::default()),
    }
  }
}
