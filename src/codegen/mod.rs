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

mod json;
mod util;

impl Context {
    fn codegen_union_class(
        &self,
        buf: &mut String,
        library: &Library,
        union: &Union,
    ) -> std::fmt::Result {
        let discrimminant = union
            .json_discrimminant
            .as_ref()
            .map(|spanned| spanned.value.as_str())
            .unwrap_or("type");

        if let Some(docs) = &union.docs {
            self.write_doc_comment(buf, docs)?;
        }

        let modifiers = match union.sealed {
            None => "abstract final",
            Some(_) => "sealed",
        };

        write!(buf, "{modifiers} class {} ", union.name)?;

        braced(buf, |out| {
            writeln!(out, "const {}();", union.name)?;

            writeln!(out)?;

            writeln!(out, "Map<String, dynamic> toJson(); ")?;
            writeln!(
                out,
                r#"factory {}.fromJson(Map<String, dynamic> json) => switch (json["{discrimminant}"]) {{"#,
                union.name,
            )?;

            for class in &union.classes {
                let name = &class.name;
                writeln!(out, r#""{name}" => {name}.fromJson(json),"#)?;
            }
            writeln!(
                out,
                r#"final other => throw ArgumentError("unknown discrimminant: $other"),"#
            )?;

            writeln!(out, "}};")?;

            Ok(())
        })?;

        for class in &union.classes {
            self.codegen_immutable_class(buf, library, class, Some(union))?;
            self.codegen_mutable_class(buf, library, class)?;
        }

        Ok(())
    }

    fn codegen_immutable_class(
        &self,
        buf: &mut String,
        library: &Library,
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

            writeln!(out, "const {}({{", class.name)?;
            for field in &class.fields {
                let required_kw = if field.defaults_to.is_none() {
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
            match superclass {
                Some(_) => writeln!(out, "}}) : super();")?,
                None => writeln!(out, "}});")?,
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
                let field_needs_to_builder = library
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

            self.generate_to_json(out, library, class, superclass)?;
            self.generate_from_json(out, library, class)?;

            writeln!(out)?;

            for extra in &class.extra_dart {
                // TODO(cameron): proper error handling
                let text = self.read_path_or_string(extra).unwrap();
                writeln!(out, "{text}")?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn codegen_mutable_class(
        &self,
        buf: &mut String,
        library: &Library,
        class: &Class,
    ) -> std::fmt::Result {
        // TODO: allow configuring
        let builder_name = format!("{}Builder", class.name);

        write!(buf, "final class {builder_name}",)?;

        braced(buf, |out| {
            for field in &class.fields {
                // TODO: more robust handling of types
                let field_needs_build = library
                    .classes
                    .iter()
                    .any(|c| c.name.as_str() == field.ty.as_str());

                let ty_name = if field_needs_build {
                    format!("{}Builder", field.ty)
                } else {
                    field.ty.to_string()
                };

                writeln!(out, "{ty_name} {};", field.name)?;
            }

            writeln!(out)?;

            writeln!(out, "{builder_name}({{")?;
            for field in &class.fields {
                writeln!(out, "required this.{},", field.name)?;
            }
            writeln!(out, "}});")?;

            writeln!(out)?;

            writeln!(out, "{0} build() => {0}(", class.name)?;
            for field in &class.fields {
                // TODO: more robust handling of types
                let field_needs_build = library
                    .classes
                    .iter()
                    .any(|c| c.name.as_str() == field.ty.as_str());

                let name = field.name.as_str();
                let build = if field_needs_build { ".build()" } else { "" };
                writeln!(out, "{name}: {name}{build},")?;
            }
            writeln!(out, ");")?;

            Ok(())
        })?;

        Ok(())
    }

    fn write_doc_comment(&self, buf: &mut String, source: &str) -> std::fmt::Result {
        let mut lines: VecDeque<_> = source.lines().collect();
        while let Some(line) = lines.pop_front()
            && line.trim().is_empty()
        {}
        while let Some(line) = lines.pop_back()
            && line.trim().is_empty()
        {}

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

pub fn codegen(ctx: Context, out: &mut impl std::io::Write) -> Result<()> {
    let mut buf = String::new();

    if let Some(preamble) = &ctx.library.preamble {
        let text = ctx.read_path_or_string(preamble)?;
        writeln!(buf, "{text}").into_diagnostic()?;
    }

    writeln!(buf, "import \"package:equatable/equatable.dart\";").into_diagnostic()?;

    for class in &ctx.library.classes {
        ctx.codegen_immutable_class(&mut buf, &ctx.library, class, None)
            .into_diagnostic()?;
        ctx.codegen_mutable_class(&mut buf, &ctx.library, class)
            .into_diagnostic()?;
    }

    for union in &ctx.library.unions {
        ctx.codegen_union_class(&mut buf, &ctx.library, union)
            .into_diagnostic()?;
    }

    let formatted = dart_format(buf)?;
    out.write_all(formatted.as_bytes()).into_diagnostic()?;

    Ok(())
}
