use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    ops::Deref,
};

use convert_case::Casing;
use knus::{DecodeScalar, ast::Value, span::Span, traits::ErrorSpan};

use crate::codegen::format_dart_literal_const;

use super::*;

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

impl Library {
    pub fn all_classes(&self) -> impl Iterator<Item = &Class> {
        self.classes
            .iter()
            .chain(self.unions.iter().flat_map(|union| &union.classes))
    }

    pub fn class_and_union_names(&self) -> impl Iterator<Item = &SpannedScalar<String>> {
        let class_names = self.all_classes().map(|class| &class.name);
        let union_names = self.unions.iter().map(|union| &union.name);

        class_names.chain(union_names)
    }

    pub fn type_names(&self) -> impl Iterator<Item = &SpannedScalar<String>> {
        let enum_names = self.enums.iter().map(|enums| &enums.name);

        self.class_and_union_names().chain(enum_names)
    }

    pub fn all_fields(&self) -> impl Iterator<Item = &Field> {
        self.all_classes().flat_map(|class| &class.fields)
    }

    pub fn all_raw_values(&self) -> impl Iterator<Item = &Value<Span>> {
        self.all_fields().flat_map(|field| &field.defaults_to)
    }

    pub fn type_has_builder(&self, type_name: &str) -> bool {
        self.class_and_union_names()
            .any(|name| name.as_str() == type_name)
    }

    pub fn json_key_for<'lib>(&self, class: &'lib Class, field: &'lib Field) -> Cow<'lib, str> {
        if let Some(key) = &field.json_key {
            return Cow::Borrowed(key);
        }

        let key = &field.name;

        let rename_case = class.json_key_case.as_ref().or(self
            .defaults
            .as_ref()
            .and_then(|d| d.field.as_ref().and_then(|f| f.json_key_case.as_ref())));

        match rename_case {
            None => Cow::Borrowed(key),
            Some(case) => Cow::Owned(key.to_case(case.value.into())),
        }
    }

    pub fn discriminant_value_for<'lib>(
        &'lib self,
        union: &'lib Union,
        class: &'lib Class,
    ) -> Cow<'lib, str> {
        if let Some(value) = &class.json_discriminant_value {
            return Cow::Owned(format_dart_literal_const(value));
        }

        let name = match union.json_discriminant_value_case {
            Some(case) => class.name.to_case(case.value.into()),
            None => class.name.to_string(),
        };

        Cow::Owned(format!("\"{name}\""))
    }
}
