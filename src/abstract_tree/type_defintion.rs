use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct TypeDefintion {
    r#type: Type,
}

impl AbstractTree for TypeDefintion {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        Error::expect_syntax_node(source, "type_definition", node)?;

        let type_node = node.child(1).unwrap();
        let r#type = Type::from_syntax_node(source, type_node)?;

        Ok(TypeDefintion { r#type })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        self.r#type.run(source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.r#type.expected_type(context)
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
    Table,
}

impl TypeDefintion {
    pub fn check(&self, value: &Value, context: &Map) -> Result<()> {
        match (&self.r#type, value) {
            (Type::Any, _)
            | (Type::Boolean, Value::Boolean(_))
            | (Type::Empty, Value::Empty)
            | (Type::Float, Value::Float(_))
            | (Type::Integer, Value::Integer(_))
            | (Type::Map, Value::Map(_))
            | (Type::Number, Value::Integer(_))
            | (Type::Number, Value::Float(_))
            | (Type::String, Value::String(_))
            | (Type::Table, Value::Table(_)) => Ok(()),
            (Type::List(_), Value::List(list)) => {
                if let Some(first) = list.items().first() {
                    self.check(first, context)
                } else {
                    Ok(())
                }
            }
            (
                Type::Function {
                    parameter_types,
                    return_type,
                },
                Value::Function(function),
            ) => {
                let parameter_type_count = parameter_types.len();
                let parameter_count = function.parameters().len();

                if parameter_type_count != parameter_count
                    || return_type.as_ref() != &function.body().expected_type(context)?
                {
                    return Err(Error::TypeCheck {
                        expected: self.r#type.clone(),
                        actual: value.clone(),
                    });
                }

                Ok(())
            }
            (Type::Boolean, _) => Err(Error::TypeCheck {
                expected: Type::Boolean,
                actual: value.clone(),
            }),
            (Type::Empty, _) => Err(Error::TypeCheck {
                expected: Type::Empty,
                actual: value.clone(),
            }),
            (Type::Float, _) => Err(Error::TypeCheck {
                expected: Type::Float,
                actual: value.clone(),
            }),
            (Type::Function { .. }, _) => Err(Error::TypeCheck {
                expected: self.r#type.clone(),
                actual: value.clone(),
            }),
            (Type::Integer, _) => Err(Error::TypeCheck {
                expected: Type::Integer,
                actual: value.clone(),
            }),
            (Type::List(_), _) => Err(Error::TypeCheck {
                expected: self.r#type.clone(),
                actual: value.clone(),
            }),
            (Type::Map, _) => Err(Error::TypeCheck {
                expected: Type::Map,
                actual: value.clone(),
            }),
            (Type::Number, _) => Err(Error::TypeCheck {
                expected: Type::Number,
                actual: value.clone(),
            }),
            (Type::String, _) => Err(Error::TypeCheck {
                expected: Type::String,
                actual: value.clone(),
            }),
            (Type::Table, _) => Err(Error::TypeCheck {
                expected: Type::Table,
                actual: value.clone(),
            }),
        }
    }
}

impl AbstractTree for Type {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        Error::expect_syntax_node(source, "type", node)?;

        let type_node = node.child(0).unwrap();

        let r#type = match type_node.kind() {
            "any" => Type::Any,
            "bool" => Type::Boolean,
            "float" => Type::Float,
            "fn" => {
                let child_count = node.child_count();
                let mut parameter_types = Vec::new();

                for index in 1..child_count - 2 {
                    let parameter_type_node = node.child(index).unwrap();
                    let parameter_type = Type::from_syntax_node(source, parameter_type_node)?;

                    parameter_types.push(parameter_type);
                }

                let return_type_node = node.child(child_count - 1).unwrap();
                let return_type = Type::from_syntax_node(source, return_type_node)?;

                Type::Function {
                    parameter_types,
                    return_type: Box::new(return_type),
                }
            }
            "int" => Type::Integer,
            "list" => {
                let item_type_node = node.child(1).unwrap();
                let item_type = Type::from_syntax_node(source, item_type_node)?;

                Type::List(Box::new(item_type))
            }
            "map" => Type::Map,
            "str" => Type::String,
            "table" => Type::Table,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "any, bool, float, fn, int, list, map, str or table",
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
            Type::Function { .. } => write!(f, "function"),
            Type::Integer => write!(f, "integer"),
            Type::List(_) => write!(f, "list"),
            Type::Map => write!(f, "map"),
            Type::Number => write!(f, "number"),
            Type::String => write!(f, "string"),
            Type::Table => write!(f, "table"),
        }
    }
}
