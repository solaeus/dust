use std::sync::{Arc, OnceLock};

use enum_iterator::{all, Sequence};

use crate::Identifier;

pub fn all_built_in_identifiers() -> impl Iterator<Item = BuiltInIdentifier> {
    all()
}

static OPTION: OnceLock<Identifier> = OnceLock::new();
static NONE: OnceLock<Identifier> = OnceLock::new();
static SOME: OnceLock<Identifier> = OnceLock::new();
static RESULT: OnceLock<Identifier> = OnceLock::new();
static OK: OnceLock<Identifier> = OnceLock::new();
static ERROR: OnceLock<Identifier> = OnceLock::new();

#[derive(Sequence, Debug)]
pub enum BuiltInIdentifier {
    Option,
    None,
    Some,
    Result,
    Ok,
    Error,
}

impl BuiltInIdentifier {
    pub fn get(&self) -> &Identifier {
        match self {
            BuiltInIdentifier::Option => {
                OPTION.get_or_init(|| Identifier::from_raw_parts(Arc::new("Option".to_string())))
            }
            BuiltInIdentifier::None => {
                NONE.get_or_init(|| Identifier::from_raw_parts(Arc::new("None".to_string())))
            }
            BuiltInIdentifier::Some => {
                SOME.get_or_init(|| Identifier::from_raw_parts(Arc::new("Some".to_string())))
            }
            BuiltInIdentifier::Result => {
                RESULT.get_or_init(|| Identifier::from_raw_parts(Arc::new("Result".to_string())))
            }
            BuiltInIdentifier::Ok => {
                OK.get_or_init(|| Identifier::from_raw_parts(Arc::new("Ok".to_string())))
            }
            BuiltInIdentifier::Error => {
                ERROR.get_or_init(|| Identifier::from_raw_parts(Arc::new("Error".to_string())))
            }
        }
    }
}
