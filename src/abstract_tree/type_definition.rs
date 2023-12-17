use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct TypeDefinition {
    r#type: Type,
}

impl TypeDefinition {
    pub fn new(r#type: Type) -> Self {
        Self { r#type }
    }

    pub fn inner(&self) -> &Type {
        &self.r#type
    }

    pub fn take_inner(self) -> Type {
        self.r#type
    }
}

impl AbstractTree for TypeDefinition {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "type_definition", node)?;

        let type_node = node.child(1).unwrap();
        let r#type = Type::from_syntax_node(source, type_node, context)?;

        Ok(TypeDefinition { r#type })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        self.r#type.run(source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.r#type.expected_type(context)
    }
}

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.r#type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Any,
    Boolean,
    Empty,
    Float,
    Function {
        parameter_types: Vec<Type>,
        return_type: Box<Type>,
    },
    Integer,
    List(Box<Type>),
    Map,
    Number,
    String,
}

impl Type {
    pub fn check(&self, other: &Type) -> Result<()> {
        match (self, other) {
            (Type::Any, _)
            | (_, Type::Any)
            | (Type::Boolean, Type::Boolean)
            | (Type::Empty, Type::Empty)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::Map, Type::Map)
            | (Type::Number, Type::Number)
            | (Type::Number, Type::Integer)
            | (Type::Number, Type::Float)
            | (Type::Integer, Type::Number)
            | (Type::Float, Type::Number)
            | (Type::String, Type::String) => Ok(()),
            (Type::List(self_item_type), Type::List(other_item_type)) => {
                if self_item_type.check(&other_item_type).is_err() {
                    Err(Error::TypeCheck {
                        expected: self.clone(),
                        actual: other.clone(),
                    })
                } else {
                    Ok(())
                }
            }
            (
                Type::Function {
                    parameter_types: self_parameter_types,
                    return_type: self_return_type,
                },
                Type::Function {
                    parameter_types: other_parameter_types,
                    return_type: other_return_type,
                },
            ) => {
                let parameter_type_pairs = self_parameter_types
                    .iter()
                    .zip(other_parameter_types.iter());

                for (self_parameter_type, other_parameter_type) in parameter_type_pairs {
                    if self_parameter_type.check(&other_parameter_type).is_err() {
                        return Err(Error::TypeCheck {
                            expected: self.clone(),
                            actual: other.clone(),
                        });
                    }
                }

                if self_return_type.check(other_return_type).is_err() {
                    Err(Error::TypeCheck {
                        expected: self.clone(),
                        actual: other.clone(),
                    })
                } else {
                    Ok(())
                }
            }
            _ => Err(Error::TypeCheck {
                expected: self.clone(),
                actual: other.clone(),
            }),
        }
    }
}

impl AbstractTree for Type {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "type", node)?;

        let type_node = node.child(0).unwrap();

        let r#type = match type_node.kind() {
            "[" => {
                let item_type_node = node.child(1).unwrap();
                let item_type = Type::from_syntax_node(source, item_type_node, context)?;

                Type::List(Box::new(item_type))
            }
            "any" => Type::Any,
            "bool" => Type::Boolean,
            "float" => Type::Float,
            "(" => {
                let child_count = node.child_count();
                let mut parameter_types = Vec::new();

                for index in 1..child_count - 2 {
                    let child = node.child(index).unwrap();

                    if child.is_named() {
                        let parameter_type = Type::from_syntax_node(source, child, context)?;

                        parameter_types.push(parameter_type);
                    }
                }

                let final_node = node.child(child_count - 1).unwrap();
                let return_type = if final_node.is_named() {
                    Type::from_syntax_node(source, final_node, context)?
                } else {
                    Type::Empty
                };

                Type::Function {
                    parameter_types,
                    return_type: Box::new(return_type),
                }
            }
            "int" => Type::Integer,
            "map" => Type::Map,
            "num" => Type::Number,
            "str" => Type::String,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "any, bool, float, fn, int, list, map, num or str",
                    actual: type_node.kind(),
                    location: type_node.start_position(),
                    relevant_source: source[type_node.byte_range()].to_string(),
                })
            }
        };

        Ok(r#type)
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::Empty)
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::Empty)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Boolean => write!(f, "bool"),
            Type::Empty => write!(f, "empty"),
            Type::Float => write!(f, "float"),
            Type::Function {
                parameter_types,
                return_type,
            } => {
                write!(f, "(")?;

                for parameter_type in parameter_types {
                    write!(f, "{parameter_type}")?;

                    if parameter_type != parameter_types.last().unwrap() {
                        write!(f, " ")?;
                    }
                }

                write!(f, ")")?;
                write!(f, " -> {return_type}")
            }
            Type::Integer => write!(f, "int"),
            Type::List(item_type) => write!(f, "[{item_type}]"),
            Type::Map => write!(f, "map"),
            Type::Number => write!(f, "num"),
            Type::String => write!(f, "str"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluate;

    use super::*;

    #[test]
    fn simple_type_check() {
        let result = evaluate("x <bool> = 1");

        assert_eq!(
            Err(Error::TypeCheck {
                expected: Type::Boolean,
                actual: Type::Integer
            }),
            result
        );
    }

    #[test]
    fn callback_type_check() {
        let result = evaluate(
            "
            x = (fn cb <() -> bool>) <bool> {
                (cb)
            }
            (x (fn) <int> { 1 })
            ",
        );

        assert_eq!(
            Err(Error::TypeCheck {
                expected: Type::Function {
                    parameter_types: vec![],
                    return_type: Box::new(Type::Boolean),
                },
                actual: Type::Function {
                    parameter_types: vec![],
                    return_type: Box::new(Type::Integer),
                },
            }),
            result
        );
    }
}