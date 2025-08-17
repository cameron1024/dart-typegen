use std::collections::HashMap;

use convert_case::{Case, Casing};
use knus::ast::{Integer, Literal, Radix};
use miette::{Diagnostic, NamedSource, Result, Severity, SourceSpan};
use thiserror::Error;

use crate::{
    context::{Context, TyKind},
    model::Field,
};

#[cfg(test)]
mod tests;

impl Context {
    pub fn validate(&self, deny_warnings: bool) -> Result<()> {
        let errors = self.collect_errors();

        if errors.is_empty() {
            return Ok(());
        }

        if !deny_warnings
            && errors
                .iter()
                .all(|e| e.severity() == Some(Severity::Warning))
        {
            for error in errors {
                eprintln!("{error:?}");
            }
            return Ok(());
        }

        miette::bail!(MultiError { errors })
    }

    fn collect_errors(&self) -> Vec<miette::Report> {
        let mut errors = vec![];
        let source = self.named_source();

        incorrect_type_name_case(self, &mut errors, &source);
        duplicate_type_names(self, &mut errors, &source);
        duplicate_field_names(self, &mut errors, &source);
        empty_union(self, &mut errors, &source);
        field_with_both_defaults(self, &mut errors, &source);
        invalid_int_literal(self, &mut errors, &source);
        empty_enum(self, &mut errors, &source);
        json_discrimminant_non_union_class(self, &mut errors, &source);
        duplicate_json_keys(self, &mut errors, &source);
        invalid_field_types(self, &mut errors, &source);
        version_too_low(self, &mut errors, &source);

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

    #[help]
    correct_name: String,
}

fn incorrect_type_name_case(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let incorrect_class_name_case = context
        .library
        .classes
        .iter()
        .filter(|class| {
            let skip = class
                .allow_non_pascal_case
                .as_ref()
                .map(|b| b.value)
                .unwrap_or(false);

            if skip {
                false
            } else {
                !class.name.is_case(Case::Pascal)
            }
        })
        .map(|class| {
            let correct_name = format!("Try renaming it to `{}`", class.name.to_case(Case::Pascal));
            IncorrectClassNameCase {
                src: source.clone(),
                source_span: class.name.span,
                correct_name,
            }
        });

    errors.extend(incorrect_class_name_case.map(Into::into));
}

// === Duplicate class names ===

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicate type name")]
#[diagnostic(help = "Try giving it a different name")]
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
#[diagnostic(help = "Try giving it a different name")]
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
#[diagnostic(help = "Unions must contain at least one `class`")]
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
#[diagnostic(
    help = "`defaults-to` allows you to translate native KDL types to Dart. If you need something that cannot be expressed in KDL (such as class instances, collection literals)"
)]
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
#[diagnostic(
    help = "Integer literals must be either decimal (i.e. `1234`) or hexadecimal (i.e. `0x1234`)"
)]
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
#[diagnostic(help = r#"Add at least one variant, e.g. `variant "myVariant"`"#)]
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

// === Json Discriminant Non Union Class ===

#[derive(Debug, Error, Diagnostic)]
#[error("Non-union class has `json-discriminant-value`")]
#[help = "Only classes that are part of a union have a json discriminant, so this value is meaningless"]
struct JsonDiscrimimantInNonUnionClass {
    #[source_code]
    src: NamedSource<String>,

    #[label(primary, "Remove this")]
    span: SourceSpan,
}

fn json_discrimminant_non_union_class(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let errs = context.library.classes.iter().filter_map(|c| {
        let value = c.json_discriminant_value.as_ref()?;

        Some(JsonDiscrimimantInNonUnionClass {
            src: source.clone(),
            span: (*value.literal.span()).into(),
        })
    });

    errors.extend(errs.map(Into::into));
}

// === Duplicate Json Keys ===

#[derive(Debug, Error, Diagnostic)]
#[error("Multiple fields have the same json key")]
struct DuplicateJsonKeys {
    #[source_code]
    src: NamedSource<String>,

    key: String,

    #[label("first field")]
    first: SourceSpan,

    #[label("second field")]
    second: SourceSpan,
}

fn duplicate_json_keys(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    for class in context.library.all_classes() {
        for (index, first) in class.fields.iter().enumerate() {
            for second in class.fields.iter().skip(index + 1) {
                let first_key = context.library.json_key_for(class, first);
                let second_key = context.library.json_key_for(class, second);

                if first_key == second_key {
                    let error = DuplicateJsonKeys {
                        src: source.clone(),
                        key: first_key.to_string(),
                        first: json_key_span(first),
                        second: json_key_span(second),
                    };
                    errors.push(error.into());
                }
            }
        }
    }
}

fn json_key_span(field: &Field) -> SourceSpan {
    field
        .json_key
        .as_ref()
        .map(|key| key.span)
        .unwrap_or(field.name.span)
}

// === Invalid Field Types ===

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid Field Type")]
struct InvalidFieldType {
    #[source_code]
    src: NamedSource<String>,

    #[label]
    span: SourceSpan,

    #[help]
    message: &'static str,
}

fn invalid_field_types(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    for (field, (ty, parse_errors)) in context
        .library
        .all_fields()
        .map(|f| (f, context.parse_ty(&f.ty)))
    {
        errors.extend(parse_errors);
        let Some(ty) = ty else {
            let err = InvalidFieldType {
                src: source.clone(),
                span: field.ty.span,
                message: "Failed to parse type",
            };

            errors.push(err.into());
            return;
        };

        if let TyKind::Map { key, .. } = &ty.kind {
            match &key.kind {
                TyKind::Simple(s) if s == "String" => {}
                _ => {
                    let err = InvalidFieldType {
                        src: source.clone(),
                        span: key.span.into(),
                        message: "Only `String` is supported as a Map key type",
                    };

                    errors.push(err.into());
                }
            }
        }
    }
}

// === Invalid Field Types ===

#[derive(Debug, Error, Diagnostic)]
#[error(
    "This config requires `dart-typegen` version {required} (or any semver-compatible version), but the current version is {current}"
)]
#[diagnostic(severity(Warning))]
struct IncompatibleVersion {
    #[source_code]
    src: NamedSource<String>,

    #[label("Version defined here")]
    span: SourceSpan,

    required: String,
    current: String,
}

#[derive(Debug, Error, Diagnostic)]
#[error("This version is not a valid semantic version")]
struct VersionNotSemver {
    #[source_code]
    src: NamedSource<String>,

    #[label("Not valid semantic version")]
    span: SourceSpan,
}

fn version_too_low(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let min_version = context
        .library
        .meta
        .as_ref()
        .and_then(|meta| meta.version.as_ref());

    let Some(min_version) = min_version else {
        return;
    };

    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let Ok(requirement) = semver::VersionReq::parse(min_version) else {
        let err = VersionNotSemver {
            src: source.clone(),
            span: min_version.span,
        };
        errors.push(err.into());
        return;
    };

    if !requirement.matches(&current) {
        let err = IncompatibleVersion {
            src: source.clone(),
            span: min_version.span,
            required: min_version.to_string(),
            current: env!("CARGO_PKG_VERSION").to_string(),
        };

        errors.push(err.into());
    }
}
