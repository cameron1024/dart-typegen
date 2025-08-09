use super::*;

impl Context {
    pub(super) fn codegen_mutable_class(
        &self,
        buf: &mut String,
        class: &Class,
    ) -> std::fmt::Result {
        // TODO: allow configuring
        let builder_name = format!("{}Builder", class.name);

        self.write_doc_comment(buf, &format!("Builder class for [{}]", class.name))?;
        write!(buf, "final class {builder_name}",)?;

        braced(buf, |out| {
            for field in &class.fields {
                // TODO: more robust handling of types
                let field_needs_build = self
                    .library
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

            if class.fields.is_empty() {
                writeln!(out, "{builder_name}();")?;
            } else {
                writeln!(out, "{builder_name}({{")?;
                for field in &class.fields {
                    writeln!(out, "required this.{},", field.name)?;
                }
                writeln!(out, "}});")?;
            }

            writeln!(out)?;

            writeln!(out, "{0} build() => {0}(", class.name)?;
            for field in &class.fields {
                // TODO: more robust handling of types
                let field_needs_build = self
                    .library
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
}
