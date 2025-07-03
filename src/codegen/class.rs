use crate::model::Field;

use super::*;

pub fn generate_class<W: Write>(class: &Class, out: &mut W) -> Result<()> {
    generate_class_impl(false, &class.name, &class.fields, "", out)?;

    Ok(())
}

fn generate_class_impl<W: Write>(
    mutable: bool,
    class_name: &str,
    fields: &[Field],
    extra: &str,
    out: &mut W,
) -> Result<()> {
    writeln!(out, "final class {class_name} with EquatableMixin {{").into_diagnostic()?;

    for field in fields {
        let name = &field.name;
        let ty = match &field.ty {
            Type::Ident(s) => s,
        };

        let final_kw = match mutable {
            true => "",
            false => "final ",
        };
        writeln!(out, "{final_kw}{ty} {name};").into_diagnostic()?;
    }

    generate_constructor(mutable, class_name, fields, &mut *out)?;


    writeln!(out, "}}").into_diagnostic()?;

    Ok(())
}

fn generate_constructor<W: Write>(
    mutable: bool,
    class_name: &str,
    fields: &[Field],
    mut out: W,
) -> Result<()> {
    let const_kw = match mutable {
        true => "",
        false => "const ",
    };
    writeln!(out, "{const_kw}{class_name} ({{").into_diagnostic()?;

    for field in fields {
        writeln!(out, "required this.{}", field.name).into_diagnostic()?;
    }

    writeln!(out, "}});").into_diagnostic()?;

    Ok(())
}
