use super::*;

impl Context {
    pub(super) fn generate_to_json(
        &self,
        buf: &mut String,
        class: &Class,
        superclass: Option<&Union>,
    ) -> std::fmt::Result {
        if superclass.is_some() {
            writeln!(buf, "@override")?;
        }
        writeln!(buf, "Map<String, dynamic> toJson() => {{")?;

        for field in &class.fields {
            let json_key = self.library.json_key_for(class, field);
            let field_name = &field.name;

            if let Some(to_json) = &field.to_json {
                writeln!(buf, "\"{json_key}\": ({to_json})({field_name}),")?;
            } else {
                let needs_to_json = self
                    .library
                    .classes
                    .iter()
                    .any(|c| c.name.as_str() == field.ty.as_str());
                let to_json = match needs_to_json {
                    false => "",
                    true => ".toJson()",
                };

                writeln!(buf, "\"{json_key}\": {field_name}{to_json},")?;
            }
        }

        if let Some(union) = superclass {
            let discriminant_key = self.library.discriminant_key_for(union);
            let discriminant_value = self.library.discriminant_value_for(union, class);

            writeln!(buf, "\"{discriminant_key}\": {discriminant_value}",)?;
        }

        writeln!(buf, "}};")?;

        Ok(())
    }

    pub(super) fn generate_from_json(&self, buf: &mut String, class: &Class) -> std::fmt::Result {
        writeln!(
            buf,
            "factory {0}.fromJson(Map<String, dynamic> json) => {0}(",
            class.name
        )?;

        for field in &class.fields {
            let json_key = self.library.json_key_for(class, field);
            let field_ty = &field.ty;
            let field_name = &field.name;

            write!(buf, "{field_name}: ")?;

            // an explicit `from-json` overrides everything
            if let Some(from_json) = &field.from_json {
                writeln!(buf, "({from_json})(json[\"{json_key}\"])")?;
            } else {
                let needs_from_json = self
                    .library
                    .classes
                    .iter()
                    .any(|c| c.name.as_str() == field.ty.as_str());

                if needs_from_json {
                    write!(buf, "{field_ty}.fromJson(")?;
                }
                write!(buf, "json[\"{json_key}\"]")?;

                if needs_from_json {
                    write!(buf, " as Map<String, dynamic>)")?;
                } else {
                    write!(buf, "as {field_ty}")?;
                }
            }
            writeln!(buf, ",")?;
        }

        writeln!(buf, ");")?;

        Ok(())
    }
}
