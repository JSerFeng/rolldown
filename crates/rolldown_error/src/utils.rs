use std::{borrow::Cow, path::Path};

use sugar_path::SugarPath;

use crate::CWD;

pub fn format_quoted_strings_with_verbs(
  list: &[impl AsRef<str>],
  verb: Option<(&str, &str)>,
) -> String {
  debug_assert!(!list.is_empty());
  let is_single_item = list.len() == 1;
  let mut quoted_list = list
    .iter()
    .map(|item| format!("\"{}\"", item.as_ref()))
    .collect::<Vec<_>>();
  let mut output = if is_single_item {
    quoted_list.into_iter().next().unwrap()
  } else {
    let last_item = quoted_list.pop().unwrap();
    format!("{} and {}", quoted_list.join(", "), last_item)
  };
  if let Some((verb, verb_past)) = verb {
    output += &format!(" {}", if is_single_item { verb } else { verb_past });
  }
  output
}

pub fn format_quoted_strings(list: &[impl AsRef<str>]) -> String {
  format_quoted_strings_with_verbs(list, None)
}

pub trait PathExt {
  fn may_display_relative(&self) -> Cow<str>;
}

impl PathExt for Path {
  fn may_display_relative(&self) -> Cow<str> {
    let path = if CWD.is_set() && self.is_absolute() {
      CWD.with(|cwd| self.relative(cwd))
    } else {
      return self.to_string_lossy();
    };
    Cow::Owned(path.display().to_string())
  }
}
