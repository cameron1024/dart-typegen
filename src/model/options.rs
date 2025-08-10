use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Defaults {
    #[knus(child, unwrap(argument))]
    pub generate_to_string: Option<SpannedScalar<bool>>,

    #[knus(child)]
    pub union: Option<UnionOptions>,

    #[knus(child)]
    pub field: Option<FieldOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct UnionOptions {
    #[knus(child, unwrap(argument))]
    pub sealed: Option<SpannedScalar<bool>>,

    #[knus(child, unwrap(argument))]
    pub json_discriminant: Option<SpannedScalar<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct FieldOptions {
    #[knus(child, unwrap(argument))]
    pub json_key_case: Option<SpannedScalar<RenameCase>>,

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DecodeScalar)]
pub enum RenameCase {
    Camel,
    Pascal,
    Snake,
    Kebab,
    ScreamingSnake,
}
