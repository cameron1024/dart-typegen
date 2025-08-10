#[macro_export]
macro_rules! test_file {
    ($name:ident) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/",
            stringify!($name),
            ".kdl"
        )
    };
}

#[macro_export]
macro_rules! failing_test_file {
    ($name:ident) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/failing/",
            stringify!($name),
            ".kdl"
        )
    };
}

#[macro_export]
macro_rules! all_test_files {
    ($macro_name:ident) => {
        $macro_name!(class_docs);
        $macro_name!(class_extra_dart);
        $macro_name!(class_field_default);
        $macro_name!(class_field_docs);
        $macro_name!(class_simple);
        $macro_name!(class_with_class_field);
        $macro_name!(default_field_case);
        $macro_name!(default_sealed_unions);
        $macro_name!(empty);
        $macro_name!(enum_docs);
        $macro_name!(enum_json_value);
        $macro_name!(enum_simple);
        $macro_name!(postamble);
        $macro_name!(preamble);
        $macro_name!(union_docs);
        $macro_name!(union_json_discriminant);
        $macro_name!(union_sealed);
        $macro_name!(union_simple);
    };
}

#[macro_export]
macro_rules! all_failing_test_files {
    ($macro_name:ident) => {};
}

mod equivalence;
mod snapshots;
