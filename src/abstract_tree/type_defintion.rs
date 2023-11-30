use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum TypeDefinition {
    Any,
    Boolean,
    Empty,
    Float,
    Function {
        parameter_types: Vec<TypeDefinition>,
        return_type: Box<TypeDefinition>,
    },
    Integer,
    List(Box<TypeDefinition>),
    Map,
    Number,
    String,
    Table,
}

impl TypeDefinition {
    //     pub fn check(&self, value: &Value) -> Result<()> {
    //         match (self, value.r#type()?) {
    //             (Type::Any, _)
    //             | (Type::Boolean, Type::Boolean)
    //             | (Type::Empty, Type::Empty)
    //             | (Type::Float, Type::Float)
    //             | (Type::Integer, Type::Integer)
    //             | (Type::Map, Type::Map)
    //             | (Type::Number, Type::Number)
    //             | (Type::Number, Type::Integer)
    //             | (Type::Number, Type::Float)
    //             | (Type::Integer, Type::Number)
    //             | (Type::Float, Type::Number)
    //             | (Type::String, Type::String)
    //             | (Type::Table, Type::Table) => Ok(()),
    //             (Type::List(expected), Type::List(actual)) => {
    //                 if expected != &actual {
    //                     Err(Error::TypeCheck {
    //                         expected: Type::List(expected.clone()),
    //                         actual: value.clone(),
    //                     })
    //                 } else {
    //                     Ok(())
    //                 }
    //             }
    //             (
    //                 Type::Function {
    //                     parameter_types: left_parameters,
    //                     return_type: left_return,
    //                 },
    //                 Type::Function {
    //                     parameter_types: right_parameters,
    //                     return_type: right_return,
    //                 },
    //             ) => {
    //                 if left_parameters != &right_parameters || left_return != &right_return {
    //                     Err(Error::TypeCheck {
    //                         expected: Type::Function {
    //                             parameter_types: left_parameters.clone(),
    //                             return_type: left_return.clone(),
    //                         },
    //                         actual: value.clone(),
    //                     })
    //                 } else {
    //                     Ok(())
    //                 }
    //             }
    //             (Type::Boolean, _) => Err(Error::TypeCheck {
    //                 expected: Type::Boolean,
    //                 actual: value.clone(),
    //             }),
    //             (Type::Empty, _) => Err(Error::TypeCheck {
    //                 expected: Type::Empty,
    //                 actual: value.clone(),
    //             }),
    //             (Type::Float, _) => Err(Error::TypeCheck {
    //                 expected: Type::Float,
    //                 actual: value.clone(),
    //             }),
    //             (expected, _) => Err(Error::TypeCheck {
    //                 expected: expected.clone(),
    //                 actual: value.clone(),
    //             }),
    //             (Type::Integer, _) => Err(Error::TypeCheck {
    //                 expected: Type::Integer,
    //                 actual: value.clone(),
    //             }),
    //             (expected, _) => Err(Error::TypeCheck {
    //                 expected: expected.clone(),
    //                 actual: value.clone(),
    //             }),
    //             (Type::Map, _) => Err(Error::TypeCheck {
    //                 expected: Type::Map,
    //                 actual: value.clone(),
    //             }),
    //             (Type::String, _) => Err(Error::TypeCheck {
    //                 expected: Type::String,
    //                 actual: value.clone(),
    //             }),
    //             (Type::Table, _) => Err(Error::TypeCheck {
    //                 expected: Type::Table,
    //                 actual: value.clone(),
    //             }),
    //         }
    //     }
}

impl AbstractTree for TypeDefinition {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        Error::expect_syntax_node(source, "type_definition", node)?;

        let type_node = node.child(1).unwrap();
        let type_symbol = &source[type_node.byte_range()];

        let r#type = match type_symbol {
            "any" => TypeDefinition::Any,
            "bool" => TypeDefinition::Boolean,
            "float" => TypeDefinition::Float,
            "fn" => {
                todo!()
            }
            "int" => TypeDefinition::Integer,
            "map" => TypeDefinition::Map,
            "str" => TypeDefinition::String,
            "table" => TypeDefinition::Table,
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

    fn expected_type(&self, _context: &Map) -> Result<TypeDefinition> {
        Ok(TypeDefinition::Empty)
    }
}

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeDefinition::Any => write!(f, "any"),
            TypeDefinition::Boolean => write!(f, "bool"),
            TypeDefinition::Empty => write!(f, "empty"),
            TypeDefinition::Float => write!(f, "float"),
            TypeDefinition::Function { .. } => write!(f, "function"),
            TypeDefinition::Integer => write!(f, "integer"),
            TypeDefinition::List(_) => write!(f, "list"),
            TypeDefinition::Map => write!(f, "map"),
            TypeDefinition::Number => write!(f, "number"),
            TypeDefinition::String => write!(f, "string"),
            TypeDefinition::Table => write!(f, "table"),
        }
    }
}
