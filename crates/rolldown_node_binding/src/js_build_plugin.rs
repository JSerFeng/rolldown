use derivative::Derivative;
use rolldown_plugin::{BuildPlugin, PluginName, ResolvedId};

use crate::{
  js_callbacks::{ResolveIdCallback, TransformCallback},
  options::BuildPluginOption,
  utils::NapiErrorExt,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct JsBuildPlugin {
  pub name: String,
  #[derivative(Debug = "ignore")]
  transform_cb: Option<TransformCallback>,
  #[derivative(Debug = "ignore")]
  resolve_id_cb: Option<ResolveIdCallback>,
}

impl JsBuildPlugin {
  pub fn new(option: BuildPluginOption) -> napi::Result<Self> {
    let transform_tsfn = option
      .transform
      .as_ref()
      .map(TransformCallback::new)
      .transpose()?;

    let resolve_id_cb = option
      .resolve_id
      .as_ref()
      .map(ResolveIdCallback::new)
      .transpose()?;

    Ok(JsBuildPlugin {
      name: option.name,
      transform_cb: transform_tsfn,
      resolve_id_cb,
    })
  }

  pub fn new_boxed(option: BuildPluginOption) -> napi::Result<Box<dyn BuildPlugin>> {
    Ok(Box::new(Self::new(option)?))
  }
}

#[async_trait::async_trait]
impl BuildPlugin for JsBuildPlugin {
  fn name(&self) -> PluginName {
    std::borrow::Cow::Borrowed(&self.name)
  }

  async fn transform(
    &self,
    _ctx: &mut rolldown_plugin::Context,
    args: &mut rolldown_plugin::TransformArgs,
  ) -> rolldown_plugin::TransformReturn {
    if let Some(cb) = &self.transform_cb {
      let res = cb
        .call_async((args.code.to_string(), args.id.to_string()))
        .await;
      res.map_err(|e| e.into_bundle_error())
    } else {
      Ok(None)
    }
  }

  async fn resolve(
    &self,
    _ctx: &mut rolldown_plugin::Context,
    args: &mut rolldown_plugin::ResolveArgs,
  ) -> rolldown_plugin::ResolveReturn {
    if let Some(cb) = &self.resolve_id_cb {
      let cb_ret = cb
        .call_async((
          args.specifier.to_string(),
          args.importer.map(|s| s.to_string()),
        ))
        .await
        .map_err(|e| e.into_bundle_error())?;

      Ok(cb_ret.map(|cb_ret| ResolvedId {
        id: cb_ret.id,
        external: cb_ret.external,
      }))
    } else {
      Ok(None)
    }
  }
}
