use std::{collections::VecDeque, fmt::Write, path::PathBuf};

use knus::{
    ast::{Decimal, Integer, Literal, Radix, Value},
    span::Span,
};
use miette::{IntoDiagnostic, Result};

use crate::{codegen::util::braced, context::Context, model::*};

pub use util::dart_format;

mod enumeration;
mod equals;
mod immutable;
mod json;
mod mutable;
mod union;
mod util;

impl Context {
    pub fn codegen(&self) -> Result<(String, Option<&SpannedScalar<PathBuf>>)> {
        let mut buf = String::new();

        writeln!(buf, "// ignore_for_file: unnecessary_cast").into_diagnostic()?;

        if let Some(preamble) = &self.library.preamble {
            writeln!(buf, "{preamble}").into_diagnostic()?;
        }

        for class in &self.library.classes {
            self.codegen_immutable_class(&mut buf, class, None)
                .into_diagnostic()?;
            self.codegen_mutable_class(&mut buf, class, None)
                .into_diagnostic()?;
        }

        for union in &self.library.unions {
            self.codegen_union_class(&mut buf, union)
                .into_diagnostic()?;
        }

        for e in &self.library.enums {
            self.codegen_enum(&mut buf, e).into_diagnostic()?;
        }

        if let Some(postamble) = &self.library.postamble {
            writeln!(buf, "{postamble}").into_diagnostic()?;
        }

        let lang_version = self
            .library
            .defaults
            .as_ref()
            .and_then(|d| d.dart_format_language_version.as_ref())
            .map(|v| v.as_str());

        let output = dart_format(buf, lang_version)?;
        let path = self.library.output.as_ref().and_then(|o| o.path.as_ref());

        Ok((output, path))
    }

    fn write_doc_comment(&self, buf: &mut String, source: &str) -> std::fmt::Result {
        let mut lines: VecDeque<_> = source.lines().collect();
        while let Some(s) = lines.front()
            && s.trim().is_empty()
        {
            lines.pop_front();
        }
        while let Some(s) = lines.back()
            && s.trim().is_empty()
        {
            lines.pop_back();
        }

        for line in lines {
            writeln!(buf, "/// {line}")?;
        }

        Ok(())
    }
}

pub fn format_dart_literal_const(defaults_to: &Value<Span>) -> String {
    match &*defaults_to.literal {
        Literal::Null => "null".into(),
        Literal::Bool(true) => "true".into(),
        Literal::Bool(false) => "false".into(),
        Literal::Int(Integer(radix, str)) => {
            let prefix = match radix {
                Radix::Bin | Radix::Oct => unreachable!("checked in validate"),
                Radix::Dec => "",
                Radix::Hex => "0x",
            };

            format!("{prefix}{str}")
        }
        Literal::Decimal(Decimal(str)) => str.to_string(),
        // TODO: deal with escaping
        Literal::String(str) => format!("\"{str}\""),
    }
}
