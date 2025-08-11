use super::*;

impl Context {
    pub(super) fn codegen_immutable_class(
        &self,
        buf: &mut String,
        class: &Class,
        superclass: Option<&Union>,
    ) -> std::fmt::Result {
        if let Some(source) = &class.docs {
            self.write_doc_comment(buf, source)?;
        }
        if let Some(annotations) = &class.annotations {
            writeln!(buf, "{annotations}")?;
        }
        if let Some(annotations) = &self
            .library
            .defaults
            .as_ref()
            .and_then(|d| d.class.as_ref())
            .and_then(|c| c.annotations.as_ref())
        {
            writeln!(buf, "{annotations}")?;
        }
        write!(buf, "final class {} ", class.name)?;
        if let Some(superclass) = &superclass {
            write!(buf, "extends {} ", superclass.name)?;
        }

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

            let builder_name = format!("{}Builder", class.name);

            writeln!(out, "{builder_name} toBuilder() => {builder_name}(")?;
            for field in &class.fields {
                let needs_to_builder = self.library.type_has_builder(&field.ty);
                let name = field.name.as_str();
                let to_builder = if needs_to_builder { ".toBuilder()" } else { "" };
                writeln!(out, "{name}: {name}{to_builder},")?;
            }
            writeln!(out, ");")?;

            writeln!(out)?;

            self.generate_to_json(out, class, superclass)?;
            self.generate_from_json(out, class)?;

            writeln!(out)?;

            let generate_to_string = self
                .library
                .defaults
                .as_ref()
                .and_then(|d| d.generate_to_string.as_ref())
                .map(|g| g.value)
                .unwrap_or(true);

            if generate_to_string {
                self.generate_to_string_enum(out, class)?;
            }

            let generate_equals = self
                .library
                .defaults
                .as_ref()
                .and_then(|d| d.generate_equals.as_ref())
                .map(|g| g.value)
                .unwrap_or(true);

            if generate_equals {
                self.generate_equals_and_hash(out, class)?;
            }

            for extra in &class.extra_dart {
                writeln!(out, "{extra}")?;
                writeln!(out)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn generate_to_string_enum(&self, buf: &mut String, class: &Class) -> std::fmt::Result {
        writeln!(buf, "@override\nString toString() => \"{}(\"", class.name)?;
        for (index, field) in class.fields.iter().enumerate() {
            let name = &field.name;
            let trailing_comma = if index == class.fields.len() - 1 {
                ""
            } else {
                ", "
            };
            writeln!(buf, "\"{name}: ${name}{trailing_comma}\"")?;
        }
        writeln!(buf, "\")\";")?;
        Ok(())
    }
}
