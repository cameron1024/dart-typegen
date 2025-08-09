use std::{
    fmt::Write,
    process::{Command, Stdio},
};

use miette::{IntoDiagnostic, bail};

use crate::model::{Library, Union};

impl Library {
    pub(super) fn discriminant_for<'lib>(&'lib self, union: &'lib Union) -> &'lib str {
        union
            .json_discriminant
            .as_ref()
            .or_else(|| {
                self.defaults
                    .as_ref()
                    .and_then(|d| d.union.as_ref()?.json_discriminant.as_ref())
            })
            .map(|spanned| spanned.value.as_str())
            .unwrap_or("type")
    }

    pub(super) fn is_sealed(&self, union: &Union) -> bool {
        union
            .sealed
            .as_ref()
            .or_else(|| {
                self.defaults
                    .as_ref()
                    .and_then(|d| d.union.as_ref()?.sealed.as_ref())
            })
            .map(|spanned| spanned.value)
            .unwrap_or(false)
    }
}

/// Run `dart format` on a string
pub fn dart_format(dart: String) -> miette::Result<String> {
    use std::io::Write;
    let mut process = Command::new("dart")
        .arg("format")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .into_diagnostic()?;

    let stdin = process.stdin.as_mut().unwrap();
    stdin.write_all(dart.as_bytes()).into_diagnostic()?;

    let output = process.wait_with_output().into_diagnostic()?;
    if !output.status.success() {
        eprintln!("invalid source:");
        eprintln!("{dart}");
        bail!("dart format failed");
    }
    let output = String::from_utf8(output.stdout).into_diagnostic()?;

    Ok(output)
}

pub fn braced<W: Write>(
    out: &mut W,
    f: impl FnOnce(&mut W) -> std::fmt::Result,
) -> std::fmt::Result {
    writeln!(out, "{{")?;
    f(out)?;
    writeln!(out, "}}")?;

    Ok(())
}

#[test]
fn dart_format_works() {
    let unformatted = "
        class Foo


        {


        }
        ";

    let formatted = dart_format(unformatted.to_string()).unwrap();
    assert_eq!(formatted.trim(), "class Foo {}");
}
