use crate::context::{Ty, TyKind};

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

            self.generate_builder_factory(out, class)?;

            let builder_name = format!("{}Builder", class.name);

            writeln!(out, "{builder_name} toBuilder() => {builder_name}(")?;

            for field in &class.fields {
                let name = &field.name;

                write!(out, "{name}: ")?;
                self.write_to_builder_expr(out, name, &self.parse_ty(&field.ty).0.unwrap())?;
                writeln!(out, ",")?;
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

            if let Some(extra) = &class.extra_dart {
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

    fn generate_builder_factory(&self, buf: &mut String, class: &Class) -> std::fmt::Result {
        let class_name = &class.name;

        if class.fields.is_empty() {
            writeln!(
                buf,
                "static {class_name}Builder builder() => {class_name}Builder();"
            )?;
            return Ok(());
        }

        writeln!(buf, "static {class_name}Builder builder({{")?;
        for field in &class.fields {
            let required_kw = if field.defaults_to.is_none() && field.defaults_to_dart.is_none() {
                "required"
            } else {
                ""
            };

            let field_ty = &field.ty;
            let field_name = &field.name;

            write!(buf, "{required_kw} {field_ty} {field_name}")?;
            match (&field.defaults_to, &field.defaults_to_dart) {
                (Some(_), Some(_)) => unreachable!("checked in validation"),
                (None, None) => {}
                (Some(defaults_to), None) => {
                    let dart = format_dart_literal_const(defaults_to);
                    if dart != "null" {
                        writeln!(buf, " = {dart}")?;
                    }
                }
                (None, Some(defaults_to_dart)) => {
                    if &**defaults_to_dart != "null" {
                        writeln!(buf, " = {defaults_to_dart}")?;
                    }
                }
            }
            writeln!(buf, ",")?;
        }
        writeln!(buf, "}}) => {class_name}Builder(")?;
        for field in &class.fields {
            let field_name = &field.name;

            write!(buf, "{field_name}: ")?;
            self.write_to_builder_expr(buf, field_name, &self.parse_ty(&field.ty).0.unwrap())?;
            writeln!(buf, ",")?;
        }
        writeln!(buf, ");")?;

        Ok(())
    }

    fn write_to_builder_expr(&self, buf: &mut String, expr: &str, ty: &Ty) -> std::fmt::Result {
        match &ty.kind {
            TyKind::Simple(ident) if self.library.type_has_builder(ident) => {
                write!(buf, "{expr}.toBuilder()")?;
            }
            TyKind::Simple(_) => {
                write!(buf, "{expr}")?;
            }
            TyKind::List(inner) => {
                write!(buf, "{expr}.map((elem) => ")?;
                self.write_to_builder_expr(buf, "elem", inner)?;
                write!(buf, ").toList()")?;
            }
            TyKind::Set(inner) => {
                write!(buf, "{expr}.map((elem) => ")?;
                self.write_to_builder_expr(buf, "elem", inner)?;
                write!(buf, ").toSet()")?;
            }
            TyKind::Map { value, .. } => {
                write!(buf, "{expr}.map((key, value) => MapEntry(key, ")?;
                self.write_to_builder_expr(buf, "value", value)?;
                write!(buf, "))")?;
            }
            TyKind::Nullable(inner) => {
                write!(buf, "{expr} == null ? null : ")?;
                self.write_to_builder_expr(buf, &format!("({expr} as {inner})"), inner)?;
            }
        }

        Ok(())
    }
}
