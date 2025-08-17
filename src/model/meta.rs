use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode)]
pub struct Meta {
    #[knus(child, unwrap(argument))]
    pub version: Option<SpannedScalar<String>>,
}
