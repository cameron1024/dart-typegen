use crate::context::Context;

macro_rules! output_snapshot {
    ($name:ident) => {
        #[test]
        fn $name() {
            let context = Context::from_str(include_str!(crate::test_file!($name))).unwrap();
            let output = context.codegen_to_string().unwrap();

            insta::assert_snapshot!(stringify!($name), output, "dart");
        }
    };
}

crate::all_test_files!(output_snapshot);
