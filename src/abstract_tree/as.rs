use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Expression, Format, List, SourcePosition, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct As {
    expression: Expression,
    r#type: Type,
    position: SourcePosition,
}

impl AbstractTree for As {
    fn from_syntax(node: Node, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("as", node)?;

        let expression_node = node.child(0).unwrap();
        let expression = Expression::from_syntax(expression_node, source, context)?;

        let type_node = node.child(2).unwrap();
        let r#type = Type::from_syntax(type_node, source, context)?;

        Ok(As {
            expression,
            r#type,
            position: SourcePosition::from(node.range()),
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(self.r#type.clone())
    }

    fn validate(&self, _source: &str, context: &Context) -> Result<(), ValidationError> {
        let initial_type = self.expression.expected_type(context)?;

        if let Type::ListOf(item_type) = &self.r#type {
            match &initial_type {
                Type::ListOf(expected_item_type) => {
                    if !item_type.accepts(&expected_item_type) {
                        return Err(ValidationError::TypeCheck {
                            expected: self.r#type.clone(),
                            actual: initial_type.clone(),
                            position: self.position,
                        });
                    }
                }
                Type::String => {
                    if let Type::String = item_type.as_ref() {
                    } else {
                        return Err(ValidationError::ConversionImpossible {
                            initial_type,
                            target_type: self.r#type.clone(),
                        });
                    }
                }

                Type::Any => {
                    // Do no validation when converting from "any" to a list.
                    // This effectively defers to runtime behavior, potentially
                    // causing a runtime error.
                }
                _ => {
                    return Err(ValidationError::ConversionImpossible {
                        initial_type,
                        target_type: self.r#type.clone(),
                    })
                }
            }
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let value = self.expression.run(source, context)?;
        let converted_value = if let Type::ListOf(_) = self.r#type {
            match value {
                Value::List(list) => Value::List(list),
                Value::String(string) => {
                    let chars = string
                        .chars()
                        .map(|char| Value::String(char.to_string()))
                        .collect();

                    Value::List(List::with_items(chars))
                }
                _ => {
                    return Err(RuntimeError::ConversionImpossible {
                        from: value.r#type()?,
                        to: self.r#type.clone(),
                        position: self.position.clone(),
                    });
                }
            }
        } else if let Type::Integer = self.r#type {
            match value {
                Value::Integer(integer) => Value::Integer(integer),
                Value::Float(float) => Value::Integer(float as i64),
                _ => {
                    return Err(RuntimeError::ConversionImpossible {
                        from: value.r#type()?,
                        to: self.r#type.clone(),
                        position: self.position.clone(),
                    })
                }
            }
        } else {
            todo!()
        };

        Ok(converted_value)
    }
}

impl Format for As {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
