use std::{
    fmt::{Debug, Display},
    ops::Deref,
    path::PathBuf,
};

use knus::{Decode, DecodeScalar, ast::Value, errors::DecodeError, span::Span, traits::ErrorSpan};
use miette::SourceSpan;

use crate::model::{Class, Field, Library};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpannedScalar<T> {
    pub value: T,
    pub span: miette::SourceSpan,
}

impl<T: DecodeScalar<S>, S: ErrorSpan> DecodeScalar<S> for SpannedScalar<T> {
    fn type_check(
        type_name: &Option<knus::span::Spanned<knus::ast::TypeName, S>>,
        ctx: &mut knus::decode::Context<S>,
    ) {
        T::type_check(type_name, ctx);
    }

    fn raw_decode(
        value: &knus::span::Spanned<knus::ast::Literal, S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        let span: miette::SourceSpan = value.span().to_owned().into();

        let value = T::raw_decode(value, ctx)?;

        Ok(Self { value, span })
    }
}

impl<T> Deref for SpannedScalar<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Display for SpannedScalar<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringOrPath {
    String(String),
    Path(PathBuf),
}

impl<S> Decode<S> for StringOrPath
where
    S: Debug + Clone + Send + Sync + 'static + Into<SourceSpan>,
{
    fn decode_node(
        node: &knus::ast::SpannedNode<S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, knus::errors::DecodeError<S>> {
        #[derive(Decode)]
        struct _StringOrPath {
            #[knus(argument)]
            text: Option<String>,
            #[knus(property)]
            path: Option<PathBuf>,
        }

        let result = match _StringOrPath::decode_node(node, ctx)? {
            _StringOrPath {
                text: Some(text),
                path: None,
            } => StringOrPath::String(text),
            _StringOrPath {
                text: None,
                path: Some(path),
            } => StringOrPath::Path(path),
            _ => {
                return Err(DecodeError::Custom(
                    "You must provide either String argument or a `path=\"...\" property but not both "
                        .into(),
                ));
            }
        };

        Ok(result)
    }
}

impl Library {
    pub fn all_classes(&self) -> impl Iterator<Item = &Class> {
        self.classes
            .iter()
            .chain(self.unions.iter().flat_map(|union| &union.classes))
    }
    pub fn type_names(&self) -> impl Iterator<Item = &SpannedScalar<String>> {
        let class_names = self.all_classes().map(|class| &class.name);
        let union_names = self.unions.iter().map(|union| &union.name);

        class_names.chain(union_names)
    }

    pub fn all_fields(&self) -> impl Iterator<Item = &Field> {
        self.all_classes().flat_map(|class| &class.fields)
    }

    pub fn all_raw_values(&self) -> impl Iterator<Item = &Value<Span>> {
        self.all_fields().flat_map(|field| &field.defaults_to)
    }
}
