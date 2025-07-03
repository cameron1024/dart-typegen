use std::{fmt::Display, ops::Deref};

use knus::{DecodeScalar, traits::ErrorSpan};

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
