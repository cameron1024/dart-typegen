use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result};

use crate::model::Library;

pub struct Context {
    pub path: PathBuf,
    pub text: String,
    pub library: Library,
}

impl Context {
    pub fn from_path(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).into_diagnostic()?;
        let parsed = Library::parse_file(path)?;

        Ok(Context {
            path: path.to_path_buf(),
            text,
            library: parsed,
        })
    }
}
