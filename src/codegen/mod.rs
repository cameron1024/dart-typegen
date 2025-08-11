use std::{collections::VecDeque, fmt::Write};

use knus::{
    ast::{Decimal, Integer, Literal, Radix, Value},
    span::Span,
};
use miette::{IntoDiagnostic, Result};

use crate::{codegen::util::braced, context::Context, model::*};

pub use util::dart_format;

mod enumeration;
mod immutable;
mod json;
mod mutable;
mod union;
mod util;

impl Context {
    #[cfg(test)]
    pub fn codegen_to_string(&self) -> Result<String> {
        let mut buf = Vec::new();
        self.codegen(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    pub fn codegen(&self, out: &mut impl std::io::Write) -> Result<()> {
        let mut buf = String::new();

        if let Some(preamble) = &self.library.preamble {
            writeln!(buf, "{preamble}").into_diagnostic()?;
        }

        writeln!(buf, "import \"package:equatable/equatable.dart\";").into_diagnostic()?;

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

        let formatted = dart_format(buf)?;
        out.write_all(formatted.as_bytes()).into_diagnostic()?;

        Ok(())
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
