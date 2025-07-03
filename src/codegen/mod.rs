use std::io::Write;

use miette::{IntoDiagnostic, Result};


mod class;

pub fn generate<W: Write>(lib: &Library, mut out: W) -> Result<()> {
    generate_preamble(lib, &mut out)?;

    for class in &lib.classes {
        generate_class(class, &mut out)?;
    }

    Ok(())
}

fn generate_preamble<W: Write>(lib: &Library, out: &mut W) -> Result<()> {
    writeln!(out, "{}", lib.config.preamble).into_diagnostic()?;
    writeln!(out, "// GENERATED FILE - DO NOT MODIY BY HAND").into_diagnostic()?;

    Ok(())
}

fn generate_class<W: Write>(class: &Class, out: &mut W) -> Result<()> {
    let class_name = &class.name;
    writeln!(out, "final class {class_name} with EquatableMixin {{").into_diagnostic()?;

    for field in &class.fields {
        let name = &field.name;
        let ty = match &field.ty {
            Type::Ident(s) => s,
        };

        writeln!(out, "final {ty} {name};").into_diagnostic()?;
    }

    generate_constructor(class, &mut out)?;

    writeln!(out, "}}").into_diagnostic()?;

    Ok(())
}

fn generate_constructor<W: Write>(class: &Class, out: &mut W) -> Result<()> {
    writeln!(out, "const {} ({{", class.name).into_diagnostic()?;

    for field in &class.fields {
        writeln!(out, "required this.{}", field.name).into_diagnostic()?;
    }

    writeln!(out, "}});").into_diagnostic()?;

    Ok(())
}

fn generate_builder<W: Write>(class: &Class, out: &mut W) -> Result<()> {
    Ok(())
}
