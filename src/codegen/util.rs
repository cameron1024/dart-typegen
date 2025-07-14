use std::{
    fmt::Write,
    process::{Command, Stdio},
};

use miette::IntoDiagnostic;

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
