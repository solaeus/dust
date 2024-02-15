use std::sync::OnceLock;

use enum_iterator::{all, Sequence};

use crate::Identifier;

pub fn all_built_in_identifiers() -> impl Iterator<Item = BuiltInIdentifier> {
    all()
}

const OPTION: OnceLock<Identifier> = OnceLock::new();
const NONE: OnceLock<Identifier> = OnceLock::new();

#[derive(Sequence)]
pub enum BuiltInIdentifier {
    Option,
    None,
}

impl BuiltInIdentifier {
    pub fn get(&self) -> Identifier {
        match self {
            BuiltInIdentifier::Option => OPTION.get_or_init(|| Identifier::new("Option")).clone(),
            BuiltInIdentifier::None => NONE.get_or_init(|| Identifier::new("None")).clone(),
        }
    }
}
