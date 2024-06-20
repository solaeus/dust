use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{context::Context, error::ValidationError, identifier::Identifier};

use super::{SourcePosition, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TypeConstructor {
    Enum(WithPosition<EnumTypeConstructor>),
    Function(WithPosition<FunctionTypeConstructor>),
    Invokation(TypeInvokationConstructor),
    List(WithPosition<ListTypeConstructor>),
    ListOf(WithPosition<Box<TypeConstructor>>),
    Raw(WithPosition<Type>),
}

impl TypeConstructor {
    pub fn position(&self) -> SourcePosition {
        match self {
            TypeConstructor::Enum(WithPosition { position, .. }) => *position,
            TypeConstructor::Function(WithPosition { position, .. }) => *position,
            TypeConstructor::Invokation(TypeInvokationConstructor {
                identifier,
                type_arguments,
            }) => {
                if let Some(arguments) = type_arguments {
                    SourcePosition(
                        identifier.position.0,
                        arguments.last().unwrap().position().1,
                    )
                } else {
                    SourcePosition(identifier.position.0, identifier.position.1)
                }
            }
            TypeConstructor::List(WithPosition { position, .. }) => *position,
            TypeConstructor::ListOf(WithPosition { position, .. }) => *position,
            TypeConstructor::Raw(WithPosition { position, .. }) => *position,
        }
    }

    pub fn construct(self, context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            TypeConstructor::Invokation(TypeInvokationConstructor { identifier, .. }) => {
                let invoked_type = if let Some(r#type) = context.get_type(&identifier.node)? {
                    r#type
                } else {
                    return Err(ValidationError::VariableNotFound {
                        identifier: identifier.node,
                        position: identifier.position,
                    });
                };

                if let Type::Enum {
                    type_parameters,
                    variants,
                } = invoked_type
                {
                    let mut mapped_variants = Vec::with_capacity(variants.len());

                    for (variant_name, content) in variants {
                        mapped_variants.push((variant_name.clone(), content.clone()));
                    }

                    Type::Enum {
                        type_parameters: type_parameters.clone(),
                        variants: mapped_variants,
                    }
                } else {
                    invoked_type
                }
            }
            TypeConstructor::Enum(enum_type_constructor) => {
                let EnumTypeConstructor {
                    type_parameters,
                    variants,
                } = enum_type_constructor.node;
                let mut type_variants = Vec::with_capacity(variants.len());

                for (variant_name, constructors) in variants {
                    if let Some(constructors) = constructors {
                        let mut types = Vec::with_capacity(constructors.len());

                        for constructor in constructors {
                            let r#type = constructor.construct(context)?;

                            types.push(r#type);
                        }

                        type_variants.push((variant_name.node, Some(types)));
                    } else {
                        type_variants.push((variant_name.node, None))
                    }
                }

                Type::Enum {
                    type_parameters: type_parameters.map(|identifiers| {
                        identifiers
                            .into_iter()
                            .map(|identifier| Type::Generic {
                                identifier: identifier.node,
                                concrete_type: None,
                            })
                            .collect()
                    }),
                    variants: type_variants,
                }
            }
            TypeConstructor::Function(function_type_constructor) => {
                let FunctionTypeConstructor {
                    type_parameters: declared_type_parameters,
                    value_parameters: declared_value_parameters,
                    return_type,
                } = function_type_constructor.node;

                let type_parameters = declared_type_parameters.map(|identifiers| {
                    identifiers
                        .into_iter()
                        .map(|identifier| identifier.node)
                        .collect()
                });
                let mut value_parameters = Vec::with_capacity(declared_value_parameters.len());

                for parameter in declared_value_parameters {
                    let r#type = parameter.construct(&context)?;

                    value_parameters.push(r#type);
                }

                let return_type = Box::new(return_type.construct(&context)?);

                Type::Function {
                    type_parameters,
                    value_parameters,
                    return_type,
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
            TypeConstructor::Raw(r#type) => r#type.node,
        };

        Ok(r#type)
    }
}

impl Display for TypeConstructor {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumTypeConstructor {
    pub type_parameters: Option<Vec<WithPosition<Identifier>>>,
    pub variants: Vec<(WithPosition<Identifier>, Option<Vec<TypeConstructor>>)>,
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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeInvokationConstructor {
    pub identifier: WithPosition<Identifier>,
    pub type_arguments: Option<Vec<TypeConstructor>>,
}
