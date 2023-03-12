use rolldown_common::{Loader, ModuleId};

#[derive(Debug, Clone)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a ModuleId>,
  pub specifier: &'a str,
}

pub struct TransformArgs<'a> {
  pub id: &'a ModuleId,
  pub code: &'a String,
  pub loader: &'a mut Loader,
}

pub struct LoadArgs<'a> {
  pub id: &'a ModuleId,
}
