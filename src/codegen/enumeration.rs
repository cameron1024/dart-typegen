use super::*;

impl Context {
    pub(super) fn codegen_enum(&self, buf: &mut String, enumeration: &Enum) -> std::fmt::Result {
        let name = &enumeration.name;
        writeln!(buf, "enum {name}")?;

        braced(buf, |out| {
            for variant in &enumeration.variants {
                writeln!(out, "{},", &variant.name)?;
            }

            writeln!(out, ";")?;

            write!(
                out,
                "factory {name}.fromJson(dynamic json) => switch (json)"
            )?;

            braced(out, |out| {
                for variant in &enumeration.variants {
                    let variant = &variant.name;
                    writeln!(out, "\"{variant}\" => {name}.{variant},")?;
                }

                writeln!(
                    out,
                    "final other => throw ArgumentError(\"Unknown variant: $other\"),"
                )?;

                Ok(())
            })?;
            writeln!(out, ";")?;

            Ok(())
        })?;

        Ok(())
    }
}
