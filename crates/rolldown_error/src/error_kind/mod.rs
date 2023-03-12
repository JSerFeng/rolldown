use std::{
  fmt::Display,
  path::{Path, PathBuf},
  sync::Arc,
};

use rolldown_common::StaticStr;
use swc_core::common::SourceFile;

use crate::utils::{format_quoted_strings, PathExt};
use crate::CWD;

pub mod error_code;

#[derive(Debug)]
pub enum ErrorKind {
  // --- Aligned with rollup
  UnresolvedEntry {
    unresolved_id: PathBuf,
  },
  ExternalEntry {
    id: PathBuf,
  },
  MissingExport {
    importer: PathBuf,
    importee: PathBuf,
    missing_export: StaticStr,
  },
  AmbiguousExternalNamespaces {
    reexporting_module: PathBuf,
    used_module: PathBuf,
    binding: StaticStr,
    sources: Vec<PathBuf>,
  },
  CircularDependency(Vec<PathBuf>),
  InvalidExportOptionValue(StaticStr),
  IncompatibleExportOptionValue {
    option_value: &'static str,
    exported_keys: Vec<StaticStr>,
    entry_module: PathBuf,
  },
  ShimmedExport {
    binding: StaticStr,
    exporter: PathBuf,
  },
  CircularReexport {
    exporter: PathBuf,
    export_name: StaticStr,
  },

  UnresolvedImport {
    specifier: StaticStr,
    importer: PathBuf,
  },

  // --- Rolldown specific
  ParseJsFailed {
    source_file: Arc<SourceFile>,
    source: swc_core::ecma::parser::error::Error,
  },

  /// This error means that rolldown panics because unrecoverable error happens.
  ///
  /// This error is also used to emulate plain error `throw`ed by rollup.
  /// For `throw new Error("Errored")` in js, you can use `ErrorKind::anyhow(anyhow::format_err!("Errored"))`
  ///
  /// We also use this to replace `panic!()` in the code for graceful shutdown.
  /// But this is not recommended.
  Panic {
    source: anyhow::Error,
  },

  // --- Custom
  Napi {
    status: String,
    reason: String,
  },

  IoError(std::io::Error),
}

impl Display for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      // Aligned with rollup
      ErrorKind::UnresolvedEntry { unresolved_id } => write!(f, "Could not resolve entry module \"{}\"", unresolved_id.may_display_relative()),
      ErrorKind::ExternalEntry { id } => write!(f, "Entry module \"{}\" cannot be external.", id.may_display_relative()),
      ErrorKind::MissingExport { missing_export, importee, importer } => write!(
        f,
        r#""{missing_export}" is not exported by "{}", imported by "{}"."#,
        importee.may_display_relative(),
        importer.may_display_relative(),
      ),
      ErrorKind::AmbiguousExternalNamespaces {
        binding,
        reexporting_module,
        used_module,
        sources,
      } => write!(
        f,
        "Ambiguous external namespace resolution: \"{}\" re-exports \"{binding}\" from one of the external modules {}, guessing \"{}\".",
        reexporting_module.may_display_relative(),
        format_quoted_strings(&sources.iter().map(|p| p.may_display_relative()).collect::<Vec<_>>()),
        used_module.may_display_relative(),
      ),
      ErrorKind::CircularDependency(path) => write!(f, "Circular dependency: {}", path.iter().map(|p| p.may_display_relative()).collect::<Vec<_>>().join(" -> ")),
      ErrorKind::InvalidExportOptionValue(value) =>  write!(f, r#""output.exports" must be "default", "named", "none", "auto", or left unspecified (defaults to "auto"), received "{value}"."#),
      ErrorKind::IncompatibleExportOptionValue { option_value, exported_keys, entry_module } => {
        let mut exported_keys = exported_keys.iter().collect::<Vec<_>>();
        exported_keys.sort();
        write!(f, r#""{option_value}" was specified for "output.exports", but entry module "{}" has the following exports: {}"#, entry_module.may_display_relative(), format_quoted_strings(&exported_keys))
      }
      ErrorKind::ShimmedExport { binding, exporter } => write!(f, r#"Missing export "{binding}" has been shimmed in module "{}"."#, exporter.may_display_relative()),
      ErrorKind::CircularReexport { export_name, exporter } => write!(f, r#""{export_name}" cannot be exported from "{}" as it is a reexport that references itself."#, exporter.may_display_relative()),
      ErrorKind::UnresolvedImport { specifier, importer } => write!(f, r#"Could not resolve "{specifier}" from "{}""#, importer.may_display_relative()),
      // Rolldown specific
      ErrorKind::Panic { source } => source.fmt(f),
      ErrorKind::Napi { status, reason } => write!(f, "Napi error: {} {}", status, reason),
      ErrorKind::ParseJsFailed { source_file, .. } => {
        write!(f, "Parse failed: {}", source_file.name )
      }
      ErrorKind::IoError(e) => e.fmt(f),
    }
  }
}

impl ErrorKind {
  /// Shorten the file paths in messages by make them relative to CWD.  
  pub fn to_readable_string(&self, cwd: impl AsRef<Path>) -> String {
    let cwd = cwd.as_ref().to_path_buf();
    CWD.set(&cwd, || self.to_string())
  }

  pub fn code(&self) -> &'static str {
    match self {
      // Aligned with rollup
      ErrorKind::UnresolvedEntry { .. } => error_code::UNRESOLVED_ENTRY,
      ErrorKind::ExternalEntry { .. } => error_code::UNRESOLVED_ENTRY,
      ErrorKind::MissingExport { .. } => error_code::MISSING_EXPORT,
      ErrorKind::AmbiguousExternalNamespaces { .. } => error_code::AMBIGUOUS_EXTERNAL_NAMESPACES,
      ErrorKind::CircularDependency(_) => error_code::CIRCULAR_DEPENDENCY,
      ErrorKind::InvalidExportOptionValue(_) => error_code::INVALID_EXPORT_OPTION,
      ErrorKind::IncompatibleExportOptionValue { .. } => error_code::INVALID_EXPORT_OPTION,
      ErrorKind::ShimmedExport { .. } => error_code::SHIMMED_EXPORT,
      ErrorKind::CircularReexport { .. } => error_code::CIRCULAR_REEXPORT,
      ErrorKind::UnresolvedImport { .. } => error_code::UNRESOLVED_IMPORT,
      // Rolldown specific
      ErrorKind::Panic { .. } => error_code::PANIC,
      ErrorKind::IoError(_) => error_code::IO_ERROR,
      ErrorKind::Napi {
        status: _,
        reason: _,
      } => todo!(),
      ErrorKind::ParseJsFailed {
        source_file: _,
        source: _,
      } => todo!(),
    }
  }
}
