use std::{collections::VecDeque, fmt::Write};

use knus::{
    ast::{Decimal, Integer, Literal, Radix, Value},
    span::Span,
};
use miette::{IntoDiagnostic, Result};

use crate::{
    codegen::util::{braced, dart_format},
    context::Context,
    model::*,
};

mod enumeration;
mod json;
mod mutable;
mod union;
mod util;

impl Context {
    #[cfg(test)]
    pub fn codegen_to_string(&self) -> Result<String> {
        let mut buf = Vec::new();
        self.codegen(&mut buf)?; Ok(String::from_utf8(buf).unwrap())
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
            self.codegen_mutable_class(&mut buf, class)
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

    

    fn codegen_immutable_class(
        &self,
        buf: &mut String,
        class: &Class,
        superclass: Option<&Union>,
    ) -> std::fmt::Result {
        if let Some(source) = &class.docs {
            self.write_doc_comment(buf, source)?;
        }

        write!(buf, "final class {} ", class.name)?;
        if let Some(superclass) = &superclass {
            write!(buf, "extends {} ", superclass.name)?;
        }
        write!(buf, "with EquatableMixin ")?;

        braced(buf, |out| {
            for field in &class.fields {
                if let Some(source) = &field.docs {
                    // TODO(cameron): make this whole function return a miette result
                    self.write_doc_comment(out, source)?;
                };
                writeln!(out, "final {} {};", field.ty, field.name)?;
            }

            writeln!(out)?;

            if class.fields.is_empty() {
                writeln!(out, "const {}()", class.name)?;
            } else {
                writeln!(out, "const {}({{", class.name)?;
                for field in &class.fields {
                    let required_kw =
                        if field.defaults_to.is_none() && field.defaults_to_dart.is_none() {
                            "required"
                        } else {
                            ""
                        };

                    write!(out, "{required_kw} this.{}", field.name)?;
                    match (&field.defaults_to, &field.defaults_to_dart) {
                        (Some(_), Some(_)) => unreachable!("checked in validation"),
                        (None, None) => {}
                        (Some(defaults_to), None) => {
                            let dart = format_dart_literal_const(defaults_to);
                            if dart != "null" {
                                writeln!(out, "= {dart}")?;
                            }
                        }
                        (None, Some(defaults_to_dart)) => {
                            if &**defaults_to_dart != "null" {
                                writeln!(out, "= {defaults_to_dart}")?;
                            }
                        }
                    }
                    writeln!(out, ",")?;
                }
                writeln!(out, "}})")?;
            }

            match superclass {
                Some(_) => writeln!(out, " : super();")?,
                None => writeln!(out, ";")?,
            }

            writeln!(out)?;

            writeln!(out, "@override List<Object?> get props => [")?;
            for field in &class.fields {
                writeln!(out, "{},", field.name)?;
            }
            writeln!(out, "];")?;

            writeln!(out)?;

            // TODO: allow configuring
            let builder_name = format!("{}Builder", class.name);

            writeln!(out, "{builder_name} toBuilder() => {builder_name}(")?;
            for field in &class.fields {
                // TODO: more robust handling of types
                let field_needs_to_builder = self
                    .library
                    .classes
                    .iter()
                    .any(|c| c.name.as_str() == field.ty.as_str());

                let name = field.name.as_str();
                let to_builder = if field_needs_to_builder {
                    ".toBuilder()"
                } else {
                    ""
                };
                writeln!(out, "{name}: {name}{to_builder},")?;
            }
            writeln!(out, ");")?;

            writeln!(out)?;

            self.generate_to_json(out, class, superclass)?;
            self.generate_from_json(out, class)?;

            writeln!(out)?;

            for extra in &class.extra_dart {
                writeln!(out, "{extra}")?;
                writeln!(out)?;
            }

            Ok(())
        })?;

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

fn format_dart_literal_const(defaults_to: &Value<Span>) -> String {
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
