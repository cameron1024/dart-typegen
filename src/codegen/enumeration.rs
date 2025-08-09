use super::*;

impl Context {
    pub(super) fn codegen_enum(&self, buf: &mut String, enumeration: &Enum) -> std::fmt::Result {
        if let Some(doc) = &enumeration.docs {
            self.write_doc_comment(buf, doc)?;
        }

        let name = &enumeration.name;
        writeln!(buf, "enum {name}")?;

        braced(buf, |out| {
            for variant in &enumeration.variants {
                if let Some(doc) = &enumeration.docs {
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
                        .map(|value| format_dart_literal_const(value))
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

            write!(out, "dynamic toJson() => switch (self)")?;
            braced(out, |out| {
                for variant in &enumeration.variants {
                    let variant_name = &variant.name;
                    let value = variant
                        .json_value
                        .as_ref()
                        .map(|value| format_dart_literal_const(value))
                        .unwrap_or_else(|| format!("\"{}\"", variant.name));

                    write!(out, "{name}.{variant_name} => {value},")?;
                }

                Ok(())
            })?;
            write!(out, ";")?;
            writeln!(out)?;

            Ok(())
        })?;

        Ok(())
    }
}
