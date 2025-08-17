use std::{borrow::Cow, fmt::Display};

use chumsky::{
    extra::Err,
    label::LabelError,
    prelude::*,
    text::{Char, TextExpected},
    util::MaybeRef,
};
use knus::span::Span;
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::model::SpannedScalar;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ty {
    // Span relative to document, the type string
    pub span: Span,
    pub kind: TyKind,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TyKind::Simple(ident) => write!(f, "{ident}"),
            TyKind::List(inner) => write!(f, "List<{inner}>"),
            TyKind::Set(inner) => write!(f, "Set<{inner}>"),
            TyKind::Map { key, value } => write!(f, "Map<{key}, {value}>"),
            TyKind::Nullable(inner) => write!(f, "{inner}?"),
        }
    }
}

impl Context {
    pub fn parse_ty(&self, value: &SpannedScalar<String>) -> (Option<Ty>, Vec<miette::Report>) {
        let span_offset = value.span.offset();
        let (output, errors) = ty(span_offset).parse(value).into_output_errors();

        let errors = errors.into_iter().map(|err| ParseDartTypeError {
            src: self.named_source(),
            span: knus::span::Span(err.span().start + span_offset, err.span().end + span_offset)
                .into(),
            reason: err.reason().to_string(),
        });

        (output, errors.map(Into::into).collect())
    }

    pub fn named_source(&self) -> NamedSource<String> {
        let source_name = match &self.path {
            Some(path) => path.to_string_lossy(),
            None => Cow::Borrowed("<memory>"),
        };
        NamedSource::new(source_name, self.text.clone()).with_language("kdl")
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to parse Dart type")]
struct ParseDartTypeError {
    #[source_code]
    src: NamedSource<String>,

    #[label]
    span: SourceSpan,

    #[help]
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TyKind {
    Simple(String),
    Nullable(Box<Ty>),
    List(Box<Ty>),
    Set(Box<Ty>),
    Map { key: Box<Ty>, value: Box<Ty> },
}

fn ty<'a>(span_offset: usize) -> impl Parser<'a, &'a str, Ty, Err<Rich<'a, char>>> {
    recursive(move |ty| {
        let list = just("List")
            .then_ignore(whitespace().repeated())
            .then_ignore(just('<'))
            .then_ignore(whitespace().repeated())
            .then(ty.clone())
            .then_ignore(whitespace().repeated())
            .then_ignore(just(',').or_not())
            .then_ignore(whitespace().repeated())
            .then_ignore(just('>'))
            .map(|(_list, ty)| TyKind::List(Box::new(ty)));

        let set = just("Set")
            .then_ignore(whitespace().repeated())
            .then_ignore(just('<'))
            .then_ignore(whitespace().repeated())
            .then(ty.clone())
            .then_ignore(whitespace().repeated())
            .then_ignore(just(',').or_not())
            .then_ignore(whitespace().repeated())
            .then_ignore(just('>'))
            .map(|(_, ty)| TyKind::Set(Box::new(ty)));

        let map = just("Map")
            .then_ignore(whitespace().repeated())
            .then_ignore(just('<'))
            .then_ignore(whitespace().repeated())
            .then(ty.clone())
            .then_ignore(whitespace().repeated())
            .then_ignore(just(','))
            .then_ignore(whitespace().repeated())
            .then(ty.clone())
            .then_ignore(whitespace().repeated())
            .then_ignore(just(',').or_not())
            .then_ignore(whitespace().repeated())
            .then_ignore(just('>'))
            .map(|((_, key), value)| TyKind::Map {
                key: Box::new(key),
                value: Box::new(value),
            });

        let simple = ident().map(|s| TyKind::Simple(s.to_string()));

        let all = choice((list, set, map, simple)).map_with(move |kind, extra| {
            let span = extra.span();
            Ty {
                kind,
                span: Span(span.start + span_offset, span.end + span_offset),
            }
        });
        let nullable =
            all.then(just('?').or_not())
                .map_with(move |(inner, huh), extra| match huh {
                    None => inner,
                    Some(_) => {
                        let span = extra.span();
                        Ty {
                            kind: TyKind::Nullable(Box::new(inner)),
                            span: Span(span.start + span_offset, span.end + span_offset),
                        }
                    }
                });

        whitespace()
            .repeated()
            .ignore_then(nullable)
            .then_ignore(whitespace().repeated())
    })
}
fn whitespace<'a>() -> impl Parser<'a, &'a str, (), Err<Rich<'a, char>>> + Clone {
    choice((just(' '), just('\n'), just('\t'))).ignored()
}

/// a modified version of [`chumsky::text::ident()`] that accepts $ as a valid char
/// anywhere
fn ident<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> + Clone {
    any()
        .try_map(|c: char, span| {
            if c.is_ident_start() || c == '$' {
                Ok(c)
            } else {
                Err(LabelError::<&'a str, _>::expected_found(
                    [TextExpected::<&'a str>::IdentifierPart],
                    Some(MaybeRef::Val(c)),
                    span,
                ))
            }
        })
        .then(
            any()
                .try_map(|c: char, span| {
                    if c.is_ident_continue() || c == '$' {
                        Ok(c)
                    } else {
                        Err(LabelError::<&'a str, _>::expected_found(
                            [TextExpected::<&'a str>::IdentifierPart],
                            Some(MaybeRef::Val(c)),
                            span,
                        ))
                    }
                })
                .repeated(),
        )
        .to_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Ty {
        ty(0).parse(src).unwrap()
    }

    #[test]
    fn can_parse_types() {
        let _ = parse("Asdf");
        let _ = parse("asdf");
        let _ = parse("$asdf");
        let _ = parse("     $_123asdf     ");
        let _ = parse("List<A,>");
        let _ = parse("List    <    A    ,      >");
        let _ = parse("Map<A, B>");

        let ty = parse("Hello");
        assert_eq!(ty.kind, TyKind::Simple("Hello".to_string()));

        let ty = parse("$_Hello123$");
        assert_eq!(ty.kind, TyKind::Simple("$_Hello123$".to_string()));

        let ty = parse("Hello?");
        assert!(matches!(ty.kind, TyKind::Nullable(_)));

        // while we probably want to error here, the parser should't throw a fit if it encounters
        // this
        let ty = parse("List");
        assert_eq!(ty.kind, TyKind::Simple("List".to_string()));

        let ty = parse("Set");
        assert_eq!(ty.kind, TyKind::Simple("Set".to_string()));

        let ty = parse("Map");
        assert_eq!(ty.kind, TyKind::Simple("Map".to_string()));

        // collections
        let ty = parse("List<Hello>");
        assert!(
            matches!(ty.kind, TyKind::List(inner) if inner.kind == TyKind::Simple("Hello".to_string()))
        );
        let ty = parse("   List<   Hello    >       ");
        assert!(
            matches!(ty.kind, TyKind::List(inner) if inner.kind == TyKind::Simple("Hello".to_string()))
        );

        let ty = parse("Set<Hello,>");
        assert!(
            matches!(ty.kind, TyKind::Set(inner) if inner.kind == TyKind::Simple("Hello".to_string()))
        );

        let ty = parse("Map<$,_$123,>");
        assert!(matches!(
                ty.kind, 
                TyKind::Map {
                    key,
                    value,
                } if key.kind == TyKind::Simple("$".to_string()) 
                  && value.kind == TyKind::Simple("_$123".to_string())));
    }
}
