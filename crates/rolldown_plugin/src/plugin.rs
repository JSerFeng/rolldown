use std::{borrow::Cow, fmt::Debug};

use crate::{Context, LoadArgs, LoadOutput, ResolveArgs, TransformArgs, TransformOutput};

#[derive(Debug)]
pub struct ResolvedId {
  pub id: String,
  pub external: bool,
}

pub type ResolveReturn = rolldown_error::Result<Option<ResolvedId>>;
pub type TransformReturn = rolldown_error::Result<Option<TransformOutput>>;
pub type LoadReturn = rolldown_error::Result<Option<LoadOutput>>;
pub type PluginName<'a> = Cow<'a, str>;

#[async_trait::async_trait]
pub trait BuildPlugin: Debug + Send + Sync {
  fn name(&self) -> PluginName;

  async fn load(&self, _ctx: &mut Context, _args: &mut LoadArgs) -> LoadReturn {
    Ok(None)
  }

  async fn resolve(&self, _ctx: &mut Context, _args: &mut ResolveArgs) -> ResolveReturn {
    Ok(None)
  }

  async fn transform(&self, _ctx: &mut Context, _args: &mut TransformArgs) -> TransformReturn {
    Ok(None)
  }
}
