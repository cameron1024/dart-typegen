use super::*;

macro_rules! parse_snapshot {
    ($name:ident) => {
        #[test]
        fn $name() {
            let text = include_str!(crate::test_file!($name));
            let config = Library::parse_impl(None, text).unwrap();

            insta::assert_debug_snapshot!(config);
        }
    };
}

crate::all_test_files!(parse_snapshot);
