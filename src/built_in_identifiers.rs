use std::sync::{Arc, OnceLock};

use enum_iterator::{all, Sequence};

use crate::Identifier;

pub fn all_built_in_identifiers() -> impl Iterator<Item = BuiltInIdentifier> {
    all()
}

static OPTION: OnceLock<Identifier> = OnceLock::new();
static NONE: OnceLock<Identifier> = OnceLock::new();

#[derive(Sequence, Debug)]
pub enum BuiltInIdentifier {
    Option,
    None,
}

impl BuiltInIdentifier {
    pub fn get(&self) -> Identifier {
        match self {
            BuiltInIdentifier::Option => OPTION
                .get_or_init(|| Identifier::from_raw_parts(Arc::new("Option".to_string())))
                .clone(),
            BuiltInIdentifier::None => NONE
                .get_or_init(|| Identifier::from_raw_parts(Arc::new("None".to_string())))
                .clone(),
        }
    }
}
