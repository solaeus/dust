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
        SyntaxError::expect_syntax_node(source, "as", node)?;

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

    fn validate(&self, source: &str, context: &Context) -> Result<(), ValidationError> {
        let expected_type = self.expression.expected_type(context)?;

        if let Type::List(item_type) = &self.r#type {
            match &expected_type {
                Type::List(expected_item_type) => {
                    if !item_type.accepts(&expected_item_type) {
                        return Err(ValidationError::TypeCheck {
                            expected: self.r#type.clone(),
                            actual: expected_type.clone(),
                            position: self.position,
                        });
                    }
                }
                Type::Any => todo!(),
                Type::Boolean => todo!(),
                Type::Collection => todo!(),
                Type::Custom(_) => todo!(),
                Type::Float => todo!(),
                Type::Function {
                    parameter_types,
                    return_type,
                } => todo!(),
                Type::Integer => todo!(),
                Type::Map(_) => todo!(),
                Type::None => todo!(),
                Type::Number => todo!(),
                Type::String => todo!(),
                Type::Range => todo!(),
                Type::Option(_) => todo!(),
            }
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let value = self.expression.run(source, context)?;
        let converted_value = if let Type::List(_) = self.r#type {
            match value {
                Value::List(list) => Value::List(list),
                Value::String(string) => {
                    let chars = string
                        .chars()
                        .map(|char| Value::String(char.to_string()))
                        .collect();

                    Value::List(List::with_items(chars))
                }
                Value::Map(_) => todo!(),
                Value::Function(_) => todo!(),
                Value::Float(_) => todo!(),
                Value::Integer(_) => todo!(),
                Value::Boolean(_) => todo!(),
                Value::Range(_) => todo!(),
                Value::Option(_) => todo!(),
                Value::Structure(_) => todo!(),
            }
        } else {
            todo!()
        };

        Ok(converted_value)
    }
}

impl Format for As {
    fn format(&self, output: &mut String, indent_level: u8) {
        todo!()
    }
}
