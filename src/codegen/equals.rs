use crate::context::{Ty, TyKind};

use super::*;

impl Context {
    pub(super) fn generate_equals_and_hash(
        &self,
        buf: &mut String,
        class: &Class,
    ) -> std::fmt::Result {
        self.generate_equals(buf, class)?;
        self.generate_hash_code(buf, class)?;
        Ok(())
    }

    fn generate_equals(&self, buf: &mut String, class: &Class) -> std::fmt::Result {
        writeln!(buf, "@override\n bool operator ==(Object other)")?;
        braced(buf, |out| {
            let class_name = &class.name;

            writeln!(out, "if (identical(this, other)) {{ return true;  }}")?;
            writeln!(out, "if (other is! {class_name}) {{ return false; }}")?;
            for field in &class.fields {
                self.generate_field_equals(out, field)?;
            }
            writeln!(out, "return true;")?;

            Ok(())
        })
    }

    fn generate_field_equals(&self, buf: &mut String, field: &Field) -> std::fmt::Result {
        use TyKind::*;
        // unwrap() checked during validation
        let ty = self.parse_ty(&field.ty).0.unwrap();
        let name = &field.name;

        match ty.kind {
            Simple(_) | Nullable(_) => {
                writeln!(buf, "if ({name} != other.{name}) {{ return false; }}")
            }
            List(_) => {
                writeln!(
                    buf,
                    "if ({name}.length != other.{name}.length) {{ return false; }}"
                )?;
                writeln!(buf, "for (var i = 0; i < {name}.length; i++)")?;
                braced(buf, |out| {
                    writeln!(out, "if ({name}[i] != other.{name}[i]) {{ return false; }}")
                })
            }
            Set(_) => {
                writeln!(
                    buf,
                    "if ({name}.length != other.{name}.length) {{ return false; }}"
                )?;
                writeln!(buf, "for (final elem in {name})")?;
                braced(buf, |out| {
                    writeln!(out, "if (!other.{name}.contains(elem)) {{ return false; }}")
                })
            }
            Map { .. } => {
                writeln!(
                    buf,
                    "if ({name}.length != other.{name}.length) {{ return false; }}"
                )?;
                writeln!(buf, "for (final entry in {name}.entries)")?;
                braced(buf, |out| {
                    writeln!(
                        out,
                        "if (entry.value != other.{name}[entry.key]) {{ return false; }}"
                    )
                })
            }
        }?;

        Ok(())
    }

    fn generate_hash_code(&self, buf: &mut String, class: &Class) -> std::fmt::Result {
        writeln!(buf, "@override\n int get hashCode => Object.hashAll([")?;
        for field in &class.fields {
            let ty = self.parse_ty(&field.ty).0.unwrap();
            self.write_hash_for_field(buf, &field.name, &ty)?;
            writeln!(buf, ",")?;
        }
        writeln!(buf, "]);")?;

        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn write_hash_for_field(&self, buf: &mut String, expr: &str, ty: &Ty) -> std::fmt::Result {
        match &ty.kind {
            TyKind::Simple(_) => write!(buf, "{expr}.hashCode")?,
            TyKind::List(inner) | TyKind::Set(inner) => {
                write!(buf, "Object.hashAll({expr}.map((elem) => ")?;
                self.write_hash_for_field(buf, "elem", inner)?;
                write!(buf, "))")?
            }
            TyKind::Map { value , .. } => {
                write!(buf, "Object.hashAll({expr}.entries.expand((entry) => [entry.key, ")?;
                self.write_hash_for_field(buf, "entry.value", value)?;
                write!(buf, "]))")?
            }
            TyKind::Nullable(_) => {
                // TODO(cameron): this isn't quite right
                writeln!(buf, "{expr}?.hashCode")?
            }
        }

        Ok(())
    }
}

// int get hashCode => super.hashCode;
// bool operator ==(Object other) {
//   return super == other;
// }
