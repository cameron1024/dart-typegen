use std::collections::HashMap;

use convert_case::{Case, Casing};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use crate::{
    context::{Context, ResolvePathError},
    model::StringOrPath,
};

impl Context {
    pub fn validate(&self) -> Result<()> {
        let mut errors = vec![];
        let source =
            NamedSource::new(self.path.to_string_lossy(), self.text.clone()).with_language("kdl");

        incorrect_class_name_case(self, &mut errors, &source);
        duplicate_class_names(self, &mut errors, &source);
        duplicate_field_names(self, &mut errors, &source);
        broken_paths(self, &mut errors, &source);

        if errors.is_empty() {
            return Ok(());
        } else {
            miette::bail!(MultiError { errors })
        }
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

fn incorrect_class_name_case(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let incorrect_class_name_case = context
        .library
        .classes
        .iter()
        .filter(|class| !class.name.is_case(Case::Pascal))
        .map(|class| {
            IncorrectClassNameCase {
                src: source.clone(),
                source_span: class.name.span,
                correct_name: class.name.to_case(Case::Pascal),
            }
            .into()
        });

    errors.extend(incorrect_class_name_case);
}

// === Duplicate class names ===

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicate class name")]
#[help = "Try giving it a different name"]
struct DuplicateClassName {
    #[source_code]
    src: NamedSource<String>,
    #[label]
    source_span: SourceSpan,
}

fn duplicate_class_names(
    context: &Context,
    errors: &mut Vec<miette::Report>,
    source: &NamedSource<String>,
) {
    let mut name_counts = HashMap::<_, usize>::new();
    for class in &context.library.classes {
        *name_counts.entry(class.name.value.as_str()).or_default() += 1usize;
    }

    for class in &context.library.classes {
        if name_counts[class.name.as_str()] > 1 {
            errors.push(
                DuplicateClassName {
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
//
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
