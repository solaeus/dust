use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Block, Error, Expression, Identifier, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<Identifier>,
    body: Block,
    r#type: Type,
}

impl Function {
    pub fn new(parameters: Vec<Identifier>, body: Block, r#type: Option<Type>) -> Self {
        let r#type = r#type.unwrap_or(Type::Function {
            parameter_types: vec![Type::Any; parameters.len()],
            return_type: Box::new(Type::Any),
        });

        Self {
            parameters,
            body,
            r#type,
        }
    }

    pub fn parameters(&self) -> &Vec<Identifier> {
        &self.parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }

    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    pub fn return_type(&self) -> Result<&Type> {
        match &self.r#type {
            Type::Function {
                parameter_types: _,
                return_type,
            } => Ok(return_type.as_ref()),
            _ => todo!(),
        }
    }

    pub fn call(&self, arguments: &[Expression], source: &str, context: &Map) -> Result<Value> {
        let function_context = Map::clone_from(context)?;

        let (parameter_types, return_type) = if let Type::Function {
            parameter_types,
            return_type,
        } = &self.r#type
        {
            (parameter_types, return_type)
        } else {
            todo!()
        };

        if self.parameters.len() != arguments.len() {
            return Err(Error::ExpectedArgumentAmount {
                function_name: "",
                expected: self.parameters.len(),
                actual: arguments.len(),
            });
        }

        let parameter_argument_pairs = self
            .parameters
            .iter()
            .zip(parameter_types.iter())
            .zip(arguments.iter());

        for ((identifier, argument_type), expression) in parameter_argument_pairs {
            let value = expression.run(source, context)?;
            let value_type = value.r#type();

            match argument_type {
                Type::Any => {}
                Type::Boolean => {
                    value.as_boolean()?;
                }
                Type::Empty => {
                    value.as_empty()?;
                }
                Type::Float => {
                    value.as_float()?;
                }
                Type::Function { .. } => {
                    value.as_function()?;
                }
                Type::Integer => {
                    value.as_integer()?;
                }
                Type::List(_) => {
                    value.as_list()?;
                }
                Type::Map => {
                    value.as_map()?;
                }
                Type::Number => {
                    value.as_number()?;
                }
                Type::String => {
                    value.as_string()?;
                }
            };

            let key = identifier.inner().clone();

            function_context
                .variables_mut()?
                .insert(key, (value, value_type));
        }

        let return_value = self.body.run(source, &function_context)?;

        match return_type.as_ref() {
            Type::Any => {}
            Type::Boolean => {
                return_value.as_boolean()?;
            }
            Type::Empty => {
                return_value.as_empty()?;
            }
            Type::Float => {
                return_value.as_float()?;
            }
            Type::Function { .. } => {
                return_value.as_function()?;
            }
            Type::Integer => {
                return_value.as_integer()?;
            }
            Type::List(_) => {
                return_value.as_list()?;
            }
            Type::Map => {
                return_value.as_map()?;
            }
            Type::Number => {
                return_value.as_number()?;
            }
            Type::String => {
                return_value.as_string()?;
            }
        };

        Ok(return_value)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ parameters: {:?}, body: {:?} }}",
            self.parameters, self.body
        )
    }
}
