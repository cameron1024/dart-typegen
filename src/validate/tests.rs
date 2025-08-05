use tempdir::TempDir;

use super::*;

fn assert_error_count(count: usize, source: &str) {
    let tempdir = TempDir::new("dart-typegen-test").unwrap();
    let config_path = tempdir.path().join("config.kdl");
    std::fs::write(&config_path, source).unwrap();

    let ctx = Context::from_path(&config_path).unwrap();
    let errors = ctx.collect_errors();

    assert_eq!(errors.len(), count);
}

#[test]
fn validate_tests() {
    #[rustfmt::skip]
    assert_error_count(1, /* kdl */ r#"
        // empty union
        union "Foo" {}
    "#);
}
