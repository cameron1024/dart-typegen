use super::*;

impl Context {
    pub(super) fn codegen_mutable_class(
        &self,
        buf: &mut String,
        class: &Class,
        superclass: Option<&Union>,
    ) -> std::fmt::Result {
        let builder_name = format!("{}Builder", class.name);

        self.write_doc_comment(buf, &format!("Builder class for [{}]", class.name))?;
        write!(buf, "final class {builder_name} ",)?;
        if let Some(superclass) = &superclass {
            
            write!(buf, "extends {}Builder ", superclass.name)?;
        }

        braced(buf, |out| {
            for field in &class.fields {
                let field_needs_build = self.library.type_has_builder(&field.ty);
                let ty_name = if field_needs_build {
                    format!("{}Builder", field.ty)
                } else {
                    field.ty.to_string()
                };

                writeln!(out, "{ty_name} {};", field.name)?;
            }

            writeln!(out)?;

            if class.fields.is_empty() {
                writeln!(out, "{builder_name}()")?;
            } else {
                writeln!(out, "{builder_name}({{")?;
                for field in &class.fields {
                    writeln!(out, "required this.{},", field.name)?;
                }
                write!(out, "}})")?;
            }

            match superclass {
                Some(_) => writeln!(out, " : super();")?,
                None => writeln!(out, ";")?,
            }

            writeln!(out)?;

            writeln!(out, "{0} build() => {0}(", class.name)?;
            for field in &class.fields {
                let field_needs_build = self.library.type_has_builder(&field.ty);
                let name = field.name.as_str();
                let build = if field_needs_build { ".build()" } else { "" };
                writeln!(out, "{name}: {name}{build},")?;
            }
            writeln!(out, ");")?;

            if let Some(extra) = &class.builder_extra_dart {
                writeln!(out, "{extra}")?;
                writeln!(out)?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
