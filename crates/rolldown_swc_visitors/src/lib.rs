#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(let_chains)]
#![feature(if_let_guard)]

mod scan;
pub use scan::*;
mod remove_export_and_import;
pub use remove_export_and_import::*;
mod finalize;
pub use finalize::*;
mod resolve;
pub use resolve::*;
mod treeshake;
use swc_core::{
  common::{Span, SyntaxContext},
  ecma::visit::VisitMut,
};
pub use treeshake::*;
mod to_cjs;
pub use to_cjs::*;
mod export_mode_shimer;
pub use export_mode_shimer::*;
mod clean_ast;
pub use clean_ast::clean_ast;

struct ClearSyntaxContext;

impl VisitMut for ClearSyntaxContext {
  fn visit_mut_span(&mut self, n: &mut Span) {
    n.ctxt = SyntaxContext::empty()
  }
}
