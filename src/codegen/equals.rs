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
        // TODO: type check field to deal with lists

        let name = &field.name;
        writeln!(buf, "if (this.{name} != other.{name}) return false;")?;

        Ok(())
    }

    fn generate_hash_code(&self, buf: &mut String, class: &Class) -> std::fmt::Result {
        writeln!(buf, "@override\n int get hashCode => Object.hashAll([")?;
        for field in &class.fields {
            writeln!(buf, "{},", field.name)?;
        }
        writeln!(buf, "]);")?;

        Ok(())
    }
}

// int get hashCode => super.hashCode;
// bool operator ==(Object other) {
//   return super == other;
// }
