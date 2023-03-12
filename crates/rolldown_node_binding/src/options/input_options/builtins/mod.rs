use derivative::Derivative;
use serde::Deserialize;

mod tsconfig;
pub use tsconfig::*;

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct BuiltinsOptions {
  pub tsconfig: Option<TsConfigOptions>,
}
