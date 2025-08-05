use std::path::Path;

use knus::{ast::Value, span::Span, Decode};
use miette::IntoDiagnostic;

pub use crate::model::util::{SpannedScalar, StringOrPath};

mod util;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
#[knus(span_type = Span)]
pub struct Library {
    /// Text to append before the start of the generated file (for example, linter directives,
    /// imports, etx.)
    #[knus(child)]
    pub preamble: Option<StringOrPath>,

    /// A list of class definitions to be generated
    #[knus(children(name = "class"))]
    pub classes: Vec<Class>,

    /// A list of union definitions to be generated
    #[knus(children(name = "union"))]
    pub unions: Vec<Union>,
}

impl Library {
    pub fn parse_file(path: &Path) -> miette::Result<Self> {
        let name = Some(path.to_string_lossy());
        let text = std::fs::read_to_string(path).into_diagnostic()?;

        Self::parse_impl(name.as_deref(), &text)
    }

    pub(crate) fn parse_impl(name: Option<&str>, text: &str) -> miette::Result<Self> {
        let name = name.unwrap_or("<memory>");
        let library = knus::parse(name, text)?;

        Ok(library)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
#[knus(span_type = Span)]
pub enum Item {
    Class(Class),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
#[knus(span_type = Span)]
pub struct Class {
    #[knus(unwrap(span))]
    pub span: Span,
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(children(name = "field"))]
    pub fields: Vec<Field>,
    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<String>>,
    /// Extra text to include into the class body
    #[knus(children)]
    pub extra_dart: Vec<StringOrPath>,

}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
#[knus(span_type = Span)]
pub struct Field {
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(property(name = "type"))]
    pub ty: SpannedScalar<String>,

    #[knus(child, unwrap(argument))]
    pub defaults_to: Option<Value<Span>>,
    
    #[knus(child)]
    pub always_required: bool,

    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<String>>,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
#[knus(span_type = Span)]
pub struct Union {
    #[knus(unwrap(span))]
    pub span: Span,
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(property)]
    pub sealed: Option<SpannedScalar<bool>>,
    #[knus(child, unwrap(argument))]
    pub json_discrimminant: Option<SpannedScalar<String>>, 
    #[knus(children(name = "class"))]
    pub classes: Vec<Class>,
}
