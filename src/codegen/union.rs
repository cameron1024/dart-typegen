use super::*;

impl Context {
    pub(super) fn codegen_union_class(&self, buf: &mut String, union: &Union) -> std::fmt::Result {
        if let Some(docs) = &union.docs {
            self.write_doc_comment(buf, docs)?;
        }

        let modifiers = match self.library.is_sealed(union) {
            false => "abstract final",
            true => "sealed",
        };

        write!(buf, "{modifiers} class {} with EquatableMixin ", union.name)?;

        self.codegen_body(buf, union)?;

        for class in &union.classes {
            self.codegen_immutable_class(buf, class, Some(union))?;
            self.codegen_mutable_class(buf, class)?;
        }

        Ok(())
    }

    fn codegen_body(&self, buf: &mut String, union: &Union) -> std::fmt::Result {
        let discriminant = self.library.discriminant_for(union);

        braced(buf, |out| {
            writeln!(out, "const {}();", union.name)?;

            writeln!(out)?;

            writeln!(out, "Map<String, dynamic> toJson(); ")?;
            writeln!(
                out,
                r#"factory {}.fromJson(Map<String, dynamic> json) => switch (json["{discriminant}"]) {{"#,
                union.name,
            )?;

            for class in &union.classes {
                let name = &class.name;
                writeln!(out, r#""{name}" => {name}.fromJson(json),"#)?;
            }
            writeln!(
                out,
                r#"final other => throw ArgumentError("unknown discriminant: $other"),"#
            )?;

            writeln!(out, "}};")?;

            Ok(())
        })
    }
}
