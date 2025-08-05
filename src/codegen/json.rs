use super::*;

impl Context {
    pub(super) fn generate_to_json(
        &self,
        buf: &mut String,
        library: &Library,
        class: &Class,
    ) -> std::fmt::Result {
        writeln!(buf, "Map<String, dynamic> toJson() => {{")?;

        for field in &class.fields {
            let needs_to_json = library
                .classes
                .iter()
                .any(|c| c.name.as_str() == field.ty.as_str());
            let to_json = match needs_to_json {
                false => "",
                true => ".toJson()",
            };

            let field_name = &field.name;

            writeln!(buf, "\"{field_name}\": {field_name}{to_json},")?;
        }

        writeln!(buf, "}};")?;

        Ok(())
    }

    pub(super) fn generate_from_json(
        &self,
        buf: &mut String,
        library: &Library,
        class: &Class,
    ) -> std::fmt::Result {
        writeln!(
            buf,
            "factory {0}.fromJson(Map<String, dynamic> json) => {0}(",
            class.name
        )?;

        for field in &class.fields {
            let needs_from_json = library
                .classes
                .iter()
                .any(|c| c.name.as_str() == field.ty.as_str());

            let field_name = &field.name;
            let field_ty = &field.ty;

            write!(buf, "{field_name}: ")?;
            if needs_from_json {
                write!(buf, "{field_ty}.fromJson(")?;
            }
            write!(buf, "json[\"{field_name}\"]")?;

            if needs_from_json {
                write!(buf, " as Map<String, dynamic>)")?;
            } else {
                write!(buf, "as {field_ty}")?;
            }
            writeln!(buf, ",")?;
        }

        writeln!(buf, ");")?;

        Ok(())
    }
}
