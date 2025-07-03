use std::path::{Path, PathBuf};

use knus::{Decode, span::Span};
use miette::IntoDiagnostic;

use crate::model::util::SpannedScalar;

mod util;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Library {
    #[knus(children(name = "class"))]
    pub classes: Vec<Class>,
}

impl Library {
    pub fn parse_file(path: &Path) -> miette::Result<Self> {
        let name = Some(path.to_string_lossy());
        let text = std::fs::read_to_string(path).into_diagnostic()?;

        Self::parse_impl(name.as_deref(), &text)
    }

    pub(crate) fn parse_impl(name: Option<&str>, text: &str) -> miette::Result<Self> {
        let name = name.unwrap_or("<memory>");
        let library = knus::parse(name, text).into_diagnostic()?;

        Ok(library)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub enum Item {
    Class(Class),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Class {
    #[knus(unwrap(span))]
    pub span: Span,
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(children(name = "field"))]
    pub fields: Vec<Field>,
    /// Extra text to include into the class body
    #[knus(children(name = "extra_dart"))]
    pub extra_dart: Vec<ExtraDart>,

    /// Path to a file containing markdown-formatted docs
    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<PathBuf>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct ExtraDart {
    #[knus(argument)]
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Field {
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(property(name = "type"))]
    pub ty: SpannedScalar<String>,
}
