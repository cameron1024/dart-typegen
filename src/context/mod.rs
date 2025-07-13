use std::path::{Path, PathBuf};

use miette::{Diagnostic, IntoDiagnostic, Result};
use thiserror::Error;

use crate::model::Library;

pub struct Context {
    pub path: PathBuf,
    pub text: String,
    pub library: Library,
}

impl Context {
    pub fn from_memory(text: &str) -> Result<Self> {
        todo!()
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).into_diagnostic()?;
        let parsed = Library::parse_file(path)?;

        Ok(Context {
            path: path.to_path_buf(),
            text,
            library: parsed,
        })
    }

    pub fn resolve_path(&self, path: &Path) -> Result<PathBuf, ResolvePathError> {
        // unwrap is fine because all parent-less paths would have failed to parse the config
        let parent = self.path.parent().unwrap();

        parent
            .join(path)
            .canonicalize()
            .map_err(|io| ResolvePathError {
                path: path.to_path_buf(),
                dir: if parent == Path::new("") {
                    std::fs::canonicalize(".")
                } else {
                    parent.canonicalize()
                }
                .unwrap(),
                io,
            })
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to resolve the path `{path}` relative to `{dir}`")]
pub struct ResolvePathError {
    path: PathBuf,
    dir: PathBuf,

    #[source]
    io: std::io::Error,
}
