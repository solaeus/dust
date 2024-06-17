use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{context::Context, error::ValidationError, identifier::Identifier};

use super::{ExpectedType, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TypeConstructor {
    Function {
        type_parameters: Option<Vec<WithPosition<Identifier>>>,
        value_parameters: Vec<(WithPosition<Identifier>, Box<WithPosition<TypeConstructor>>)>,
        return_type: Box<WithPosition<TypeConstructor>>,
    },
    Identifier(WithPosition<Identifier>),
    List {
        length: usize,
        item_type: Box<WithPosition<TypeConstructor>>,
    },
    ListOf(WithPosition<Box<TypeConstructor>>),
    Type(Type),
}

impl TypeConstructor {
    pub fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        todo!()
    }

    pub fn construct(self, context: &Context) -> Result<Type, ValidationError> {
        match self {
            TypeConstructor::Function {
                type_parameters: _,
                value_parameters: _,
                return_type: _,
            } => todo!(),
            TypeConstructor::Identifier(WithPosition {
                node: identifier,
                position,
            }) => {
                if let Some(r#type) = context.get_type(&identifier)? {
                    Ok(r#type)
                } else {
                    Err(ValidationError::VariableNotFound {
                        identifier,
                        position,
                    })
                }
            }
            TypeConstructor::List { length, item_type } => {
                let constructed_type = item_type.node.construct(context)?;

                Ok(Type::List {
                    length,
                    item_type: Box::new(constructed_type),
                })
            }
            TypeConstructor::Type(r#type) => Ok(r#type),
            TypeConstructor::ListOf(_) => todo!(),
        }
    }
}

impl ExpectedType for TypeConstructor {
    fn expected_type(&self, _: &mut Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }
}

impl Display for TypeConstructor {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        todo!()
    }
}
