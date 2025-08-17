use super::*;

impl Context {
    pub(super) fn codegen_enum(&self, buf: &mut String, enumeration: &Enum) -> std::fmt::Result {
        if let Some(doc) = &enumeration.docs {
            self.write_doc_comment(buf, doc)?;
        }

        let default_annotations = self
            .library
            .defaults
            .as_ref()
            .and_then(|d| d.r#enum.as_ref())
            .and_then(|e| e.annotations.as_ref());

        if let Some(annotations) = default_annotations {
            writeln!(buf, "{annotations}")?;
        }
        if let Some(annotations) = &enumeration.annotations {
            writeln!(buf, "{annotations}")?;
        }

        let name = &enumeration.name;
        writeln!(buf, "enum {name}")?;

        braced(buf, |out| {
            for variant in &enumeration.variants {
                if let Some(doc) = &variant.docs {
                    self.write_doc_comment(out, doc)?;
                }
                writeln!(out, "{},", &variant.name)?;
            }

            writeln!(out, ";")?;

            write!(
                out,
                "factory {name}.fromJson(dynamic json) => switch (json)"
            )?;

            braced(out, |out| {
                for variant in &enumeration.variants {
                    let value = variant
                        .json_value
                        .as_ref()
                        .map(format_dart_literal_const)
                        .unwrap_or_else(|| format!("\"{}\"", variant.name));

                    let variant = &variant.name;
                    writeln!(out, "{value} => {name}.{variant},")?;
                }

                writeln!(
                    out,
                    "final other => throw ArgumentError(\"Unknown variant: $other\"),"
                )?;

                Ok(())
            })?;
            writeln!(out, ";")?;
            writeln!(out)?;

            write!(out, "dynamic toJson() => switch (this)")?;
            braced(out, |out| {
                for variant in &enumeration.variants {
                    let variant_name = &variant.name;
                    let value = variant
                        .json_value
                        .as_ref()
                        .map(format_dart_literal_const)
                        .unwrap_or_else(|| format!("\"{}\"", variant.name));

                    write!(out, "{name}.{variant_name} => {value},")?;
                }

                Ok(())
            })?;
            write!(out, ";")?;
            writeln!(out)?;

            let generate_to_string = self
                .library
                .defaults
                .as_ref()
                .and_then(|d| d.generate_to_string.as_ref())
                .map(|g| g.value)
                .unwrap_or(true);

            if generate_to_string {
                self.generate_to_string_class(out, enumeration)?;
            }

            if let Some(extra_dart) = &enumeration.extra_dart {
                writeln!(out, "{extra_dart}")?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn generate_to_string_class(&self, buf: &mut String, enumeration: &Enum) -> std::fmt::Result {
        writeln!(buf, "@override\nString toString() => switch (this)")?;
        braced(buf, |out| {
            for variant in &enumeration.variants {
                let enum_name = &enumeration.name;
                let variant_name = &variant.name;

                writeln!(out, "{enum_name}.{variant_name} => \"{variant_name}\",")?;
            }

            Ok(())
        })?;

        writeln!(buf, ";")
    }
}
