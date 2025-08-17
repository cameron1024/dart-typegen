use std::path::Path;

use knus::DecodeScalar;
use knus::{Decode, ast::Value, span::Span};
use miette::IntoDiagnostic;

pub use options::*;
pub use meta::*;
pub use util::*;

mod options;
mod meta;
mod util;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Decode)]
#[knus(span_type = Span)]
pub struct Library {
    /// Text to append before the start of the generated file (for example, linter directives,
    /// imports, etc.)
    #[knus(child, unwrap(argument))]
    pub preamble: Option<String>,
    #[knus(child, unwrap(argument))]
    pub postamble: Option<String>,
    
    #[knus(child)]
    pub meta: Option<Meta>,

    #[knus(child)]
    pub defaults: Option<Defaults>,

    #[knus(children(name = "enum"))]
    pub enums: Vec<Enum>,

    #[knus(children(name = "class"))]
    pub classes: Vec<Class>,

    /// A list of union definitions to be generated
    #[knus(children(name = "union"))]
    pub unions: Vec<Union>,
}

impl Library {
    #[cfg(test)]
    pub fn parse_memory(s: &str) -> miette::Result<Self> {
        Self::parse_impl(None, s)
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Decode)]
#[knus(span_type = Span)]
pub struct Class {
    #[knus(unwrap(span))]
    pub span: Span,
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(property)]
    pub allow_non_pascal_case: Option<SpannedScalar<bool>>,
    #[knus(children(name = "field"))]
    pub fields: Vec<Field>,
    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<String>>,
    #[knus(child, unwrap(argument))]
    pub json_key_case: Option<SpannedScalar<RenameCase>>,
    #[knus(child, unwrap(argument))]
    pub json_discriminant_value: Option<Value<Span>>,

    #[knus(child, unwrap(argument))]
    pub annotations: Option<SpannedScalar<String>>,
    #[knus(child, unwrap(argument))]
    pub builder_annotations: Option<SpannedScalar<String>>,

    /// Extra text to include into the class body
    #[knus(child, unwrap(argument))]
    pub extra_dart: Option<SpannedScalar<String>>,

    /// Extra text to include into the class body
    #[knus(child, unwrap(argument))]
    pub builder_extra_dart: Option<SpannedScalar<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Decode)]
#[knus(span_type = Span)]
pub struct Field {
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(property(name = "type"))]
    pub ty: SpannedScalar<String>,

    #[knus(child, unwrap(argument))]
    pub defaults_to: Option<Value<Span>>,

    #[knus(child, unwrap(argument))]
    pub defaults_to_dart: Option<SpannedScalar<String>>,

    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<String>>,

    #[knus(child, unwrap(argument))]
    pub to_json: Option<SpannedScalar<String>>,
    #[knus(child, unwrap(argument))]
    pub from_json: Option<SpannedScalar<String>>,

    #[knus(child, unwrap(argument))]
    pub json_key: Option<SpannedScalar<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Decode)]
#[knus(span_type = Span)]
pub struct Union {
    #[knus(unwrap(span))]
    pub span: Span,
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(property)]
    pub sealed: Option<SpannedScalar<bool>>,
    #[knus(child, unwrap(argument))]
    pub json_discriminant: Option<SpannedScalar<String>>,
    #[knus(child, unwrap(argument))]
    pub json_discriminant_value_case: Option<SpannedScalar<RenameCase>>,
    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<String>>,
    #[knus(children(name = "class"))]
    pub classes: Vec<Class>,
    #[knus(children, unwrap(argument))]
    pub extra_dart: Vec<SpannedScalar<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Decode)]
#[knus(span_type = Span)]
pub struct Enum {
    #[knus(unwrap(span))]
    pub span: Span,
    #[knus(argument)]
    pub name: SpannedScalar<String>,
    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<String>>,
    #[knus(child, unwrap(argument))]
    pub annotations: Option<SpannedScalar<String>>,
    #[knus(child, unwrap(argument))]
    pub extra_dart: Option<SpannedScalar<String>>,
    #[knus(children(name = "variant"))]
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Decode)]
#[knus(span_type = Span)]
pub struct EnumVariant {
    #[knus(argument)]
    pub name: SpannedScalar<String>,

    #[knus(child, unwrap(argument))]
    pub docs: Option<SpannedScalar<String>>,

    #[knus(child, unwrap(argument))]
    pub json_value: Option<Value<Span>>,
}
