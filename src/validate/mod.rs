use std::{borrow::Cow, collections::HashMap};

use convert_case::{Case, Casing};
use knus::ast::{Integer, Literal, Radix};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use crate::context::Context;

#[cfg(test)]
mod tests;

impl Context {
    pub fn validate(&self) -> Result<()> {
        let errors = self.collect_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            miette::bail!(MultiError { errors })
        }
    }

    fn collect_errors(&self) -> Vec<miette::Report> {
        let mut errors = vec![];
        let source_name = match &self.path {
            Some(path) => path.to_string_lossy(),
            None => Cow::Borrowed("<memory>"),
        };
        let source = NamedSource::new(source_name, self.text.clone()).with_language("kdl");

        incorrect_type_name_case(self, &mut errors, &source);
        duplicate_type_names(self, &mut errors, &source);
        duplicate_field_names(self, &mut errors, &source);
        empty_union(self, &mut errors, &source);
        field_with_both_defaults(self, &mut errors, &source);
        invalid_int_literal(self, &mut errors, &source);
        empty_enum(self, &mut errors, &source);

        errors
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Errors occurred")]
struct MultiError {
    #[related]
    errors: Vec<miette::Report>,
}

// === Incorrect class name ===

#[derive(Debug, Error, Diagnostic)]
#[error("Class had name that was not PascalCase")]
struct IncorrectClassNameCase {
    #[source_code]
    src: NamedSource<String>,
    #[label]
    source_span: SourceSpan,

    #[help = "Try renaming it to `{correct_name}`"]
    correct_name: String,
}

fn incorrect_type_name_case(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let incorrect_class_name_case = context
        .library
        .type_names()
        .filter(|name| !name.is_case(Case::Pascal))
        .map(|name| IncorrectClassNameCase {
            src: source.clone(),
            source_span: name.span,
            correct_name: name.to_case(Case::Pascal),
        });

    errors.extend(incorrect_class_name_case.map(Into::into));
}

// === Duplicate class names ===

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicate type name")]
#[help = "Try giving it a different name"]
struct DuplicateTypeName {
    #[source_code]
    src: NamedSource<String>,
    #[label]
    source_span: SourceSpan,
}

fn duplicate_type_names(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let mut name_counts = HashMap::<_, usize>::new();
    for name in context.library.type_names() {
        *name_counts.entry(name.value.as_str()).or_default() += 1usize;
    }

    for class in &context.library.classes {
        if name_counts[class.name.as_str()] > 1 {
            errors.push(
                DuplicateTypeName {
                    src: source.to_owned(),
                    source_span: class.name.span,
                }
                .into(),
            );
        }
    }
}

// === Duplicate field names ===

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicate field name")]
#[help = "Try giving it a different name"]
struct DuplicateFieldNames {
    #[source_code]
    src: NamedSource<String>,
    #[label]
    source_span: SourceSpan,
}

fn duplicate_field_names(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    for class in &context.library.classes {
        let mut name_counts = HashMap::<_, usize>::new();
        for field in &class.fields {
            *name_counts.entry(field.name.value.as_str()).or_default() += 1usize;
        }

        for field in &class.fields {
            if name_counts[field.name.as_str()] > 1 {
                errors.push(
                    DuplicateFieldNames {
                        src: source.to_owned(),
                        source_span: field.name.span,
                    }
                    .into(),
                );
            }
        }
    }
}

// === Empty union ===

#[derive(Debug, Error, Diagnostic)]
#[error("Union was empty")]
#[help = "Unions must contain at least one `class`"]
struct EmptyUnion {
    #[source_code]
    src: NamedSource<String>,
    #[label]
    source_span: SourceSpan,
}

fn empty_union(context: &Context, errors: &mut Vec<miette::Report>, source: &NamedSource<String>) {
    let errs = context
        .library
        .unions
        .iter()
        .filter(|union| union.classes.is_empty())
        .map(|union| EmptyUnion {
            src: source.clone(),
            source_span: union.span.into(),
        });

    errors.extend(errs.map(Into::into));
}

// === Empty union ===

#[derive(Debug, Error, Diagnostic)]
#[error("Field has a definition for both `defaults-to` and `defaults-to-dart`")]
#[help = "`defaults-to` allows you to translate native KDL types to Dart. If you need something
that cannot be expressed in KDL (such as class instances, collection literals)"]
struct FieldWithBothDefaults {
    #[source_code]
    src: NamedSource<String>,

    #[label = "`defaults-to` defined here"]
    defaults_to: SourceSpan,

    #[label = "`defaults-to-dart` defined here"]
    defaults_to_dart: SourceSpan,
}

fn field_with_both_defaults(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let errs = context.library.all_fields().filter_map(|field| {
        let defaults_to = field.defaults_to.as_ref()?;
        let defaults_to_dart = field.defaults_to_dart.as_ref()?;

        Some(FieldWithBothDefaults {
            src: source.clone(),
            defaults_to: (*defaults_to.literal.span()).into(),
            defaults_to_dart: defaults_to_dart.span,
        })
    });

    errors.extend(errs.map(Into::into));
}

// === Invalid Int Literals ===

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid integer literal")]
#[help = "Integer literals must be either decimal (i.e. `1234`) or hexadecimal (i.e. `0x1234`)"]
struct InvalidIntLiteral {
    #[source_code]
    src: NamedSource<String>,

    #[label = "invalid"]
    span: SourceSpan,
}

fn invalid_int_literal(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let errs = context.library.all_raw_values().filter_map(|value| {
        let Literal::Int(Integer(radix, _str)) = &*value.literal else {
            return None;
        };

        if matches!(radix, Radix::Dec | Radix::Hex) {
            return None;
        }

        Some(InvalidIntLiteral {
            src: source.clone(),
            span: (*value.literal.span()).into(),
        })
    });

    errors.extend(errs.map(Into::into));
}

// === Invalid Int Literals ===

#[derive(Debug, Error, Diagnostic)]
#[error("Enum has no variants")]
#[help = r#"Add at least one variant, e.g. `variant "myVariant"`"#]
struct EmptyEnum {
    #[source_code]
    src: NamedSource<String>,

    #[label]
    span: SourceSpan,
}

fn empty_enum(context: &Context, errors: &mut Vec<miette::Report>, source: &NamedSource<String>) {
    let errs = context
        .library
        .enums
        .iter()
        .filter(|e| e.variants.is_empty())
        .map(|e| EmptyEnum {
            src: source.clone(),
            span: e.span.into(),
        });

    errors.extend(errs.map(Into::into));
}
