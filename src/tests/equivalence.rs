//! Tests that check that two inputs produce identical outputs

use crate::context::Context;

fn assert_equivalent(left: &str, right: &str) {
    let left = Context::from_str(left)
        .unwrap()
        .codegen_to_string()
        .unwrap();
    let right = Context::from_str(right)
        .unwrap()
        .codegen_to_string()
        .unwrap();

    pretty_assertions::assert_eq!(left, right)
}

#[test]
fn default_sealed() {
    assert_equivalent(
        /* kdl */
        r#"
            union "Foo" sealed=true {
                class "X"
                class "Y"
            }
        "#,
        /* kdl */
        r#"
            defaults {
                union {
                    sealed true
                } 
            }
            union "Foo" {
                class "X"
                class "Y"
            }
        "#,
    );
}
#[test]
fn default_sealed_overridden_by_specific() {
    assert_equivalent(
        /* kdl */
        r#"
            union "Foo" sealed=false {
                class "X"
                class "Y"
            }
        "#,
        /* kdl */
        r#"
            defaults {
                union {
                    sealed true
                } 
            }
            union "Foo" sealed=false {
                class "X"
                class "Y"
            }
        "#,
    );
}

#[test]
fn default_json_discriminant() {
    assert_equivalent(
        /* kdl */
        r#"
            union "Foo" {
                json-discriminant "__type"
                class "X"
                class "Y"
            }
        "#,
        /* kdl */
        r#"
            defaults {
                union {
                    json-discriminant "__type"
                } 
            }
            union "Foo" {
                class "X"
                class "Y"
            }
        "#,
    );
}
