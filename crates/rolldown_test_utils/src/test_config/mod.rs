use std::path::Path;

use schemars::JsonSchema;
use serde::Deserialize;
mod input_options;
mod output_options;

#[macro_export]
macro_rules! impl_serde_default {
  ($name:ident) => {
    impl Default for $name {
      fn default() -> Self {
        serde_json::from_str("{}").expect("Failed to parse default config")
      }
    }
  };
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestConfig {
  #[serde(default)]
  pub input: input_options::InputOptions,
  #[serde(default)]
  pub output: output_options::OutputOptions,
  pub expected_error: Option<ExpectedError>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedError {
  pub code: String,
  pub message: String,
}

impl TestConfig {
  pub fn from_config_path(filepath: &Path) -> Self {
    let config_str = std::fs::read_to_string(filepath).expect("Failed to read test config file");
    let test_config: TestConfig =
      serde_json::from_str(&config_str).expect("Failed to parse test config file");
    test_config
  }
}
