use std::fmt::Write;

use miette::{IntoDiagnostic, Result};

use crate::{
    codegen::util::{braced, dart_format},
    model::*,
};

mod util;

struct Context {
    buffer: String,
}

impl Context {
    fn codegen_immutable_class(&mut self, class: &Class) -> std::fmt::Result {
        write!(
            self.buffer,
            "final class {} with EquatableMixin",
            class.name
        )?;

        braced(&mut self.buffer, |out| {
            for field in &class.fields {
                writeln!(out, "final {} {};", field.ty, field.name)?;
            }

            writeln!(out)?;

            writeln!(out, "const {}({{", class.name)?;
            for field in &class.fields {
                writeln!(out, "required this.{},", field.name)?;
            }
            writeln!(out, "}});")?;

            writeln!(out, "@override List<Object?> get props => [")?;
            for field in &class.fields {
                writeln!(out, "{},", field.name)?;
            }
            writeln!(out, "];")?;

            Ok(())
        })?;

        Ok(())
    }
}

pub fn codegen(out: &mut impl std::io::Write, model: &Library) -> Result<()> {
    let mut ctx = Context {
        buffer: String::new(),
    };

    for class in &model.classes {
        ctx.codegen_immutable_class(class).into_diagnostic()?;
    }

    let formatted = dart_format(ctx.buffer)?;
    out.write_all(formatted.as_bytes()).into_diagnostic()?;

    Ok(())
}
