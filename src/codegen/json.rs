use crate::context::TyKind;

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
                writeln!(buf, "\"{json_key}\": ")?;
                writeln!(buf, "// ignore: unnecessary_parenthesis")?;
                write!(buf, "({to_json})({field_name}),")?;
            } else {
                write!(buf, "\"{json_key}\": ")?;
                let ty = self.parse_ty(&field.ty).0.unwrap();

                let is_generated_by_us = self
                    .library
                    .classes
                    .iter()
                    .any(|c| c.name.as_str() == field.ty.as_str());

                if is_generated_by_us {
                    write!(buf, "{field_name}.toJson()")?;
                } else if let TyKind::Set(_) = ty.kind {
                    write!(buf, "{field_name}.toList()")?;
                } else {
                    write!(buf, "{field_name}")?;
                }

                writeln!(buf, ",")?;
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
            let expr = format!("json[\"{json_key}\"]");

            write!(buf, "{field_name}: ")?;

            // if a field has a default, we always check for null and then return the default if
            // null
            if let Some(defaults_to) = &field.defaults_to {
                let default = format_dart_literal_const(defaults_to);
                write!(buf, "{expr} == null ? {default} : ")?;
            } else if let Some(defaults_to_dart) = &field.defaults_to_dart {
                write!(buf, "{expr} == null ? {defaults_to_dart} : ")?;
            }

            if let Some(from_json) = &field.from_json {
                writeln!(buf)?;
                writeln!(buf, "// ignore: unnecessary_parenthesis")?;
                writeln!(buf, "({from_json})({expr})")?;
            } else {
                let is_generated_by_us = self
                    .library
                    .classes
                    .iter()
                    .any(|c| c.name.as_str() == field.ty.as_str());

                if is_generated_by_us {
                    write!(buf, "{field_ty}.fromJson({expr} as Map<String, dynamic>)")?;
                } else {
                    let ty = self.parse_ty(&field.ty).0.unwrap();
                    match ty.kind {
                        TyKind::Set(_) => write!(buf, "Set.from({expr} as List<dynamic>)")?,
                        _ => write!(buf, "{expr} as {}", field.ty)?,
                    }
                }
            }
            writeln!(buf, ",")?;
        }

        writeln!(buf, ");")?;

        Ok(())
    }
}
