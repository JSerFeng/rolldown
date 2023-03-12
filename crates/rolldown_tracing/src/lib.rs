use std::sync::{atomic::AtomicBool, Arc};

use tracing::{metadata::LevelFilter, Level};
use tracing_chrome::{ChromeLayerBuilder, FlushGuard};

static IS_INITIALIZE: AtomicBool = AtomicBool::new(false);

pub fn enable_tracing_on_demand() -> Option<FlushGuard> {
  use tracing_subscriber::{fmt, prelude::*, EnvFilter};
  if !IS_INITIALIZE.swap(true, std::sync::atomic::Ordering::SeqCst) {
    if std::env::var("TRACING").is_ok() {
      let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
      tracing_subscriber::registry()
        .with(chrome_layer)
        .with(
          tracing_subscriber::filter::Targets::new().with_targets(vec![("rolldown", Level::TRACE)]),
        )
        .init();
      Some(guard)
    } else {
      tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
          tracing_subscriber::filter::Targets::new().with_targets(vec![("rolldown", Level::TRACE)]),
        )
        .with(
          EnvFilter::builder()
            .with_default_directive(LevelFilter::WARN.into())
            .from_env_lossy(),
        )
        .init();
      None
    }
  } else {
    None
  }
}

#[derive(Debug, Default, Clone)]
pub struct ContextedTracer {
  context: Vec<Arc<String>>,
}

impl ContextedTracer {
  pub fn context(mut self, ctxt: String) -> Self {
    self.context.push(ctxt.into());
    self
  }

  pub fn emit_trace(&self, info: String) {
    for ctxt in &self.context {
      tracing::trace!("{}: {}", ansi_term::Color::Yellow.paint("context"), ctxt);
    }
    tracing::trace!(info)
  }
}
