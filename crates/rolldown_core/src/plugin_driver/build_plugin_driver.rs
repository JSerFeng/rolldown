use std::sync::Arc;

use rolldown_common::{Loader, ModuleId};
use rolldown_plugin::{
  BuildPlugin, Context, LoadArgs, LoadReturn, ResolveArgs, ResolveReturn, TransformArgs,
};
use tokio::sync::RwLock;

use crate::UnaryBuildResult;

pub(crate) type SharedBuildPluginDriver = Arc<RwLock<BuildPluginDriver>>;

#[derive(Debug, Default)]
pub(crate) struct BuildPluginDriver {
  pub plugins: Vec<Box<dyn BuildPlugin>>,
}

impl BuildPluginDriver {
  pub(crate) fn new(plugins: Vec<Box<dyn BuildPlugin>>) -> Self {
    Self { plugins }
  }

  pub(crate) fn into_shared(self) -> SharedBuildPluginDriver {
    Arc::new(RwLock::new(self))
  }

  pub(crate) async fn load(&self, id: &ModuleId) -> LoadReturn {
    let mut load_args = LoadArgs { id: &id };
    for plugin in &self.plugins {
      let output = plugin.load(&mut Context::new(), &mut load_args).await?;
      if output.is_some() {
        return Ok(output);
      }
    }
    Ok(None)
  }

  pub(crate) async fn resolve(&self, mut args: ResolveArgs<'_>) -> ResolveReturn {
    for plugin in &self.plugins {
      let output = plugin.resolve(&mut Context::new(), &mut args).await?;
      if output.is_some() {
        return Ok(output);
      }
    }
    Ok(None)
  }

  pub(crate) async fn transform(
    &self,
    id: &ModuleId,
    code: String,
    loader: &mut Loader,
  ) -> UnaryBuildResult<String> {
    let mut code = code;
    for plugin in &self.plugins {
      let output = plugin
        .transform(
          &mut Context::new(),
          &mut TransformArgs {
            id,
            code: &code,
            loader,
          },
        )
        .await?;
      if let Some(output) = output {
        code = output
      }
    }
    Ok(code)
  }
}
