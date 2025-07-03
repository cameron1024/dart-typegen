use crate::{codegen, model::Library};

#[test]
fn test_foo() {
    let kdl = include_str!("./foo.kdl");

    let config = Library::parse_impl(None, &kdl).unwrap();

    let mut generated = Vec::new();
    codegen::codegen(&mut generated, &config).unwrap();
    let generated = String::from_utf8(generated).unwrap();

    insta::assert_snapshot!(generated);
}
