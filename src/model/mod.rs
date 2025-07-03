use std::path::Path;

use knus::Decode;
use miette::IntoDiagnostic;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub items: Vec<Item>,
}

impl Config {
    pub fn parse_file(path: &Path) -> miette::Result<Self> {
        let name = Some(path.to_string_lossy());
        let text = std::fs::read_to_string(path).into_diagnostic()?;

        Self::parse_impl(name.as_deref(), &text)
    }

    fn parse_impl(name: Option<&str>, text: &str) -> miette::Result<Self> {
        let name = name.unwrap_or("<memory>");
        let items = knus::parse(name, text).into_diagnostic()?;

        Ok(Config { items })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub enum Item {
    Class(Class),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Class {
    #[knus(argument)]
    pub name: String,
    #[knus(children(name = "field"))]
    pub fields: Vec<Field>,
    /// Extra text to include into the class body
    #[knus(children(name = "extra_dart"))]
    pub extra_dart: Vec<ExtraDart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct ExtraDart {
    #[knus(argument)]
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Field {
    #[knus(argument)]
    pub name: String,
    #[knus(property(name = "type"))]
    pub ty: String,
}
