use super::*;

#[test]
fn can_parse_example() {
    let text = include_str!("./example.kdl");
    let config = Library::parse_impl(None, text).unwrap();

    insta::assert_debug_snapshot!(config);
}
