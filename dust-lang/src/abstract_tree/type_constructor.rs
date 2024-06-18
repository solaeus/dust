use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{context::Context, error::ValidationError, identifier::Identifier};

use super::{ExpectedType, SourcePosition, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TypeConstructor {
    Function(WithPosition<FunctionTypeConstructor>),
    Identifier(WithPosition<Identifier>),
    List(WithPosition<ListTypeConstructor>),
    ListOf(WithPosition<Box<TypeConstructor>>),
    Type(WithPosition<Type>),
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionTypeConstructor {
    pub type_parameters: Option<Vec<WithPosition<Identifier>>>,
    pub value_parameters: Vec<TypeConstructor>,
    pub return_type: Box<TypeConstructor>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ListTypeConstructor {
    pub length: usize,
    pub item_type: Box<TypeConstructor>,
}

impl TypeConstructor {
    pub fn position(&self) -> SourcePosition {
        match self {
            TypeConstructor::Function(WithPosition { position, .. }) => *position,
            TypeConstructor::Identifier(WithPosition { position, .. }) => *position,
            TypeConstructor::List(WithPosition { position, .. }) => *position,
            TypeConstructor::ListOf(WithPosition { position, .. }) => *position,
            TypeConstructor::Type(WithPosition { position, .. }) => *position,
        }
    }

    pub fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        todo!()
    }

    pub fn construct(self, context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            TypeConstructor::Function(_) => todo!(),
            TypeConstructor::Identifier(WithPosition {
                node: identifier, ..
            }) => {
                if let Some(r#type) = context.get_type(&identifier)? {
                    Type::Generic(Some(Box::new(r#type)))
                } else {
                    Type::Generic(None)
                }
            }
            TypeConstructor::List(positioned_constructor) => {
                let ListTypeConstructor { length, item_type } = positioned_constructor.node;
                let constructed_type = item_type.construct(context)?;

                Type::List {
                    length,
                    item_type: Box::new(constructed_type),
                }
            }
            TypeConstructor::ListOf(item_type) => {
                let item_type = item_type.node.construct(&context)?;

                Type::ListOf(Box::new(item_type))
            }
            TypeConstructor::Type(r#type) => r#type.node,
        };

        Ok(r#type)
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
