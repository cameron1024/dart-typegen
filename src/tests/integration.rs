use std::{fmt::Write, process::Command};

use crate::{codegen::dart_format, context::Context};

const TYPE_NAMES: &[&str] = &["TopLevel", "Animal", "Dog", "Cat", "Unused"];
const ENUM_NAMES: &[&str] = &["Color"];
const EXPRESSIONS: &[&str] = &[
    "topLevel",
    "topLevel.color",
    "topLevel.name",
    "topLevel.color",
    "topLevel.pet",
    "topLevel.secondPet",
    "topLevelAlt",
    "topLevelAlt.color",
    "topLevelAlt.name",
    "topLevelAlt.color",
    "topLevelAlt.pet",
    "topLevelAlt.secondPet",
];

const DEFAULT_TOPLEVEL: &str = /* dart */
    r#"
    final topLevel = TopLevel(
        name: "Some name",
        age: 234,
        color: Color.red,
        pet: Dog(
            name: "Alfred",
            color: Color.green,
            aliases: ["buddy", "megatron"],
        ),
        secondPet: Cat(
            name: "asshole",
            satanicPower: 15,
            data: {"hello": "world", "foo": 123, },
        ),
    );
"#;

// mostly testing default params
const ALT_TOPLEVEL: &str = /* dart */
    r#"
    final topLevelAlt = TopLevel(
        name: "Some name",
        pet: Cat(data: {}),
        
    );
"#;

fn check_equals_and_hash_code(buf: &mut String, type_name: &str) -> std::fmt::Result {
    const BODY: &str = /* dart */
        r#"
        if (obj != obj) {
            throw Exception("not equal to self");
        }

        if (obj.hashCode != obj.hashCode) {
            throw Exception("hashCode not equal");
        }
    "#;
    writeln!(buf, "void checkEquals{type_name}({type_name} obj) {{")?;
    writeln!(buf, "{BODY}")?;

    writeln!(buf, "}}")?;

    Ok(())
}

fn check_json(buf: &mut String, type_name: &str) -> std::fmt::Result {
    const BODY: &str = /* dart */
        r#"
        if (obj != decoded) {
            throw Exception("json-roundrip error");
        }

        if (obj.hashCode != decoded.hashCode) {
            throw Exception("json-roundrip hashCode error");
        }
    "#;
    writeln!(buf, "void checkJson{type_name}({type_name} obj) {{")?;

    writeln!(buf, "final encoded = obj.toJson();")?;
    writeln!(buf, "final decoded = {type_name}.fromJson(encoded);")?;
    writeln!(buf, "{BODY}")?;

    writeln!(buf, "}}")?;

    Ok(())
}

fn check_builder(buf: &mut String, type_name: &str) -> std::fmt::Result {
    const BODY: &str = /* dart */
        r#"
        final builder = obj.toBuilder();
        final rebuilt = builder.build();

        if (obj != rebuilt) {
            throw Exception("builder-roundrip error");
        }
    "#;

    writeln!(buf, "void checkBuilder{type_name}({type_name} obj) {{")?;
    writeln!(buf, "{BODY}")?;
    writeln!(buf, "}}")?;

    Ok(())
}

fn call_checks(buf: &mut String, type_name: &str, expr: &str) -> std::fmt::Result {
    writeln!(buf, "checkJson{type_name}({expr});")?;
    writeln!(buf, "checkBuilder{type_name}({expr});")?;
    Ok(())
}

fn main_fn(buf: &mut String) -> std::fmt::Result {
    writeln!(buf, "import 'generated.dart';")?;

    writeln!(buf, "void main() {{")?;

    writeln!(buf, "{DEFAULT_TOPLEVEL}")?;
    call_checks(buf, "TopLevel", "topLevel")?;
    call_checks(buf, "Animal", "topLevel.pet")?;
    call_checks(buf, "Animal", "topLevel.secondPet!")?;

    writeln!(buf, "{ALT_TOPLEVEL}")?;
    call_checks(buf, "TopLevel", "topLevelAlt")?;
    call_checks(buf, "Animal", "topLevelAlt.pet")?;

    for expr in EXPRESSIONS {
        writeln!(buf, "if ({expr} != {expr}) throw Exception('not equal');").unwrap();
    }

    writeln!(buf, "}}")?;

    Ok(())
}

fn main_dart() -> String {
    let mut buf = String::new();

    main_fn(&mut buf).unwrap();

    for name in TYPE_NAMES {
        check_equals_and_hash_code(&mut buf, name).unwrap();
        check_json(&mut buf, name).unwrap();
        check_builder(&mut buf, name).unwrap();
    }

    for name in ENUM_NAMES {
        check_json(&mut buf, name).unwrap();
    }

    buf
}

const PUBSPEC: &str = /* yaml */
    r#"
name: dart_typegen_test
environment:
    sdk: ">3.0.0"
"#;

#[test]
fn integration_test() {
    let tempdir = tempdir::TempDir::new("dart-typegen-test").unwrap();
    let package_dir = tempdir.path().join("dart_typegen_test");
    std::fs::create_dir(&package_dir).unwrap();

    println!("package_dir: {}", package_dir.to_string_lossy());

    let context = Context::from_str(include_str!(crate::test_file!(kitchen_sink))).unwrap();
    let generated = context.codegen_to_string().unwrap();
    let main = dart_format(main_dart()).unwrap();

    std::fs::write(package_dir.join("pubspec.yaml"), PUBSPEC).unwrap();
    std::fs::write(package_dir.join("generated.dart"), generated).unwrap();
    std::fs::write(package_dir.join("main.dart"), main).unwrap();

    std::mem::forget(tempdir);

    let status = Command::new("dart")
        .arg("pub")
        .arg("get")
        .current_dir(&package_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dart")
        .arg("analyze")
        .current_dir(&package_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dart")
        .arg("run")
        .arg("main.dart")
        .current_dir(&package_dir)
        .status()
        .unwrap();
    assert!(status.success());
}
