use std::collections::HashMap;

use convert_case::{Case, Casing};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use crate::{
    context::{Context, ResolvePathError},
    model::StringOrPath,
};

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

    fn collect_errors(&self) -> Vec<miette::Report>{
        let mut errors = vec![];
        let source =
            NamedSource::new(self.path.to_string_lossy(), self.text.clone()).with_language("kdl");

        incorrect_type_name_case(self, &mut errors, &source);
        duplicate_type_names(self, &mut errors, &source);
        duplicate_field_names(self, &mut errors, &source);
        broken_paths(self, &mut errors, &source);
        empty_union(self, &mut errors, &source);

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

// === Broken paths ===

#[derive(Debug, Error, Diagnostic)]
#[error("Path could not be resolved")]
#[help = "Paths are resolved relative to the directory containing the `.kdl` file"]
struct BrokenPath {
    #[source_code]
    src: NamedSource<String>,
    #[label]
    source_span: SourceSpan,

    #[source]
    #[diagnostic_source]
    resolve_error: ResolvePathError,
}

fn broken_paths(context: &Context, errors: &mut Vec<miette::Report>, source: &NamedSource<String>) {
    for class in &context.library.classes {
        let Some(string_or_path) = &class.docs else {
            continue;
        };

        let StringOrPath::Path(path) = &string_or_path.value else {
            continue;
        };

        let Err(resolve_error) = context.resolve_path(path) else {
            continue;
        };

        errors.push(
            BrokenPath {
                src: source.to_owned(),
                source_span: string_or_path.span,
                resolve_error,
            }
            .into(),
        );
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
