use std::path::{Path, PathBuf};

use miette::{Diagnostic, IntoDiagnostic, Result};
use thiserror::Error;

use crate::model::Library;

pub struct Context {
    pub path: Option<PathBuf>,
    pub text: String,
    pub library: Library,
}

impl Context {
    #[cfg(test)]
    pub fn from_str(text: &str) -> Result<Self> {
        let parsed = Library::parse_memory(text)?;

        Ok(Context {
            path: None,
            text: text.to_string(),
            library: parsed,
        })
    }
    pub fn from_path(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).into_diagnostic()?;
        let parsed = Library::parse_file(path)?;

        Ok(Context {
            path: path.to_path_buf().into(),
            text,
            library: parsed,
        })
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ReadPathError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ResolvePath(#[from] ResolvePathError),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to resolve the path `{path}` relative to `{dir}`")]
pub struct ResolvePathError {
    path: PathBuf,
    dir: PathBuf,

    #[source]
    io: std::io::Error,
}
