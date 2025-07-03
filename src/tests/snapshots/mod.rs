use std::path::Path;

use crate::{codegen, context::Context};

#[test]
fn test_foo() {
    let context = Context::from_path(Path::new("src/tests/snapshots/foo.kdl")).unwrap();

    let mut generated = Vec::new();
    codegen::codegen(context, &mut generated).unwrap();
    let generated = String::from_utf8(generated).unwrap();

    insta::assert_snapshot!(generated);
}
