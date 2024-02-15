use std::sync::OnceLock;

use enum_iterator::{all, Sequence};

use crate::{
    error::rw_lock_error::RwLockError, Context, EnumDefinition, Identifier, Type, TypeDefinition,
};

static OPTION: OnceLock<Result<TypeDefinition, RwLockError>> = OnceLock::new();
static RESULT: OnceLock<Result<TypeDefinition, RwLockError>> = OnceLock::new();

pub fn all_built_in_type_definitions() -> impl Iterator<Item = BuiltInTypeDefinition> {
    all()
}

#[derive(Sequence)]
pub enum BuiltInTypeDefinition {
    Option,
    Result,
}

impl BuiltInTypeDefinition {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInTypeDefinition::Option => "Option",
            BuiltInTypeDefinition::Result => "Result",
        }
    }

    pub fn get(&self, _context: &Context) -> &Result<TypeDefinition, RwLockError> {
        match self {
            BuiltInTypeDefinition::Option => OPTION.get_or_init(|| {
                let definition = TypeDefinition::Enum(EnumDefinition::new(
                    Identifier::new(self.name()),
                    vec![
                        (Identifier::new("Some"), Some(Type::Any)),
                        (Identifier::new("None"), None),
                    ],
                ));

                Ok(definition)
            }),
            BuiltInTypeDefinition::Result => RESULT.get_or_init(|| {
                let definition = TypeDefinition::Enum(EnumDefinition::new(
                    Identifier::new(self.name()),
                    vec![
                        (Identifier::new("Ok"), Some(Type::Any)),
                        (Identifier::new("Err"), Some(Type::Any)),
                    ],
                ));

                Ok(definition)
            }),
        }
    }
}
