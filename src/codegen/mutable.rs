use crate::context::{Ty, TyKind};

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

        let default_annotations = self
            .library
            .defaults
            .as_ref()
            .and_then(|d| d.class.as_ref())
            .and_then(|c| c.builder_annotations.as_ref());

        if let Some(annotations) = default_annotations {
            writeln!(buf, "{annotations}")?;
        }

        if let Some(annotations) = &class.builder_annotations {
            writeln!(buf, "{annotations}")?;
        }

        write!(buf, "final class {builder_name} ",)?;
        if let Some(superclass) = &superclass {
            write!(buf, "extends {}Builder ", superclass.name)?;
        }

        braced(buf, |out| {
            for field in &class.fields {
                self.write_builder_ty(out, &self.parse_ty(&field.ty).0.unwrap())?;
                writeln!(out, " {};", field.name)?;
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
                let name = &field.name;
                write!(out, "{name}: ")?;
                self.write_build_expr(out, name, &self.parse_ty(&field.ty).0.unwrap())?;
                writeln!(out, ",")?;
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

    pub(super) fn write_builder_ty(&self, buf: &mut String, ty: &Ty) -> std::fmt::Result {
        match &ty.kind {
            TyKind::Simple(ident) if self.library.type_has_builder(ident) => {
                write!(buf, "{ident}Builder")?;
            }
            TyKind::Simple(ident) => {
                write!(buf, "{ident}")?;
            }
            TyKind::List(inner) => {
                write!(buf, "List<")?;
                self.write_builder_ty(buf, inner)?;
                write!(buf, ">")?;
            }
            TyKind::Set(inner) => {
                write!(buf, "Set<")?;
                self.write_builder_ty(buf, inner)?;
                write!(buf, ">")?;
            }
            TyKind::Map { value, .. } => {
                write!(buf, "Map<String, ")?;
                self.write_builder_ty(buf, value)?;
                write!(buf, ">")?;
            }
            TyKind::Nullable(inner) => {
                self.write_builder_ty(buf, inner)?;
                write!(buf, "?")?;
            }
        }

        Ok(())
    }

    fn write_build_expr(&self, buf: &mut String, expr: &str, ty: &Ty) -> std::fmt::Result {
        match &ty.kind {
            TyKind::Simple(ident) if self.library.type_has_builder(ident) => {
                write!(buf, "{expr}.build()")?;
            }
            TyKind::Simple(_) => {
                write!(buf, "{expr}")?;
            }
            TyKind::List(inner) => {
                write!(buf, "{expr}.map((elem) => ")?;
                self.write_build_expr(buf, "elem", inner)?;
                write!(buf, ").toList()")?;
            }
            TyKind::Set(inner) => {
                write!(buf, "{expr}.map((elem) => ")?;
                self.write_build_expr(buf, "elem", inner)?;
                write!(buf, ").toSet()")?;
            }
            TyKind::Map { value, .. } => {
                write!(buf, "{expr}.map((key, value) => MapEntry(key, ")?;
                self.write_build_expr(buf, "value", value)?;
                write!(buf, "))")?;
            }
            TyKind::Nullable(inner) => {
                write!(buf, "{expr} == null ? null : ")?;
                let mut inner_builder_ty = String::new();
                self.write_builder_ty(&mut inner_builder_ty, inner)?;
                self.write_build_expr(buf, &format!("({expr} as {inner_builder_ty})"), inner)?;
            }
        }

        Ok(())
    }
}
