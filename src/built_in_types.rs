use std::sync::OnceLock;

use crate::{Identifier, Type};

static OPTION: OnceLock<Type> = OnceLock::new();

pub enum BuiltInType {
    Option(Option<Type>),
}

impl BuiltInType {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInType::Option(_) => "Option",
        }
    }

    pub fn get(&self) -> &Type {
        match self {
            BuiltInType::Option(content_type) => OPTION.get_or_init(|| {
                if let Some(content_type) = content_type {
                    Type::Custom {
                        name: Identifier::new("Option"),
                        argument: Some(Box::new(content_type.clone())),
                    }
                } else {
                    Type::Custom {
                        name: Identifier::new("Option"),
                        argument: None,
                    }
                }
            }),
        }
    }
}
