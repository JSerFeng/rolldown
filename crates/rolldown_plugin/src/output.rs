use rolldown_common::Loader;

pub type TransformOutput = String;

pub struct LoadOutput {
  pub code: String,
  pub loader: Option<Loader>,
}
