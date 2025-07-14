use std::path::Path;

use knus::{Decode, span::Span};
use miette::IntoDiagnostic;

pub use crate::model::util::{SpannedScalar, StringOrPath};

mod util;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Library {
    /// Text to append before the start of the generated file (for example, linter directives,
    /// imports, etx.)
    #[knus(child)]
    pub preamble: Option<StringOrPath>,

    /// A list of class definitions to be generated
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
    pub extra_dart: Vec<StringOrPath>,

    #[knus(unwrap(child))]
    pub docs: Option<SpannedScalar<StringOrPath>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Field {
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(property(name = "type"))]
    pub ty: SpannedScalar<String>,

    #[knus(unwrap(child))]
    pub docs: Option<SpannedScalar<StringOrPath>>,
}
