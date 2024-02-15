use std::sync::OnceLock;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

use crate::{Identifier, Type};

static OPTION: OnceLock<Type> = OnceLock::new();

#[derive(Sequence, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInType {
    Option,
}

impl BuiltInType {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInType::Option => todo!(),
        }
    }

    pub fn get(&self, inner_type: Option<Type>) -> &Type {
        match self {
            BuiltInType::Option => OPTION.get_or_init(|| {
                if let Some(inner_type) = inner_type {
                    Type::CustomWithArgument {
                        name: Identifier::new("Option"),
                        argument: Box::new(inner_type),
                    }
                } else {
                    Type::Custom(Identifier::new("Option"))
                }
            }),
        }
    }
}
