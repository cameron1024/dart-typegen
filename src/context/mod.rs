use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result};

pub struct Context {
    path: PathBuf,
    text: String,
}

impl Context {
    pub fn from_path(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).into_diagnostic()?;

        Ok(Context {
            path: path.to_path_buf(),
            text,
        })
    }
}
