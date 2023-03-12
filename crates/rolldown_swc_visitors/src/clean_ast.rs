use swc_core::{
  common::{Span, SyntaxContext},
  ecma::visit::{as_folder, Fold, VisitMut},
};

struct CleanAst;

pub fn clean_ast() -> impl Fold + VisitMut {
  as_folder(CleanAst)
}

// perf: cleaning doesn't require ordering. Maybe we could do it in parallel
impl VisitMut for CleanAst {
  fn visit_mut_span(&mut self, span: &mut Span) {
    span.ctxt = SyntaxContext::empty()
  }
}
