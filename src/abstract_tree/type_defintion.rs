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

    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    pub fn check(&self, other: &TypeDefinition, context: &Map) -> Result<()> {
        match (&self.r#type, &other.r#type) {
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
            | (Type::String, Type::String)
            | (Type::Table, Type::Table) => Ok(()),
            (Type::List(self_item_type), Type::List(other_item_type)) => {
                let self_defintion = TypeDefinition::new(self_item_type.as_ref().clone());
                let other_definition = &TypeDefinition::new(other_item_type.as_ref().clone());

                self_defintion.check(other_definition, context)
            }
            _ => Err(Error::RuntimeTypeCheck {
                expected: self.clone(),
                actual: other.clone(),
            }),
        }
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

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
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
        parameter_types: Vec<TypeDefinition>,
        return_type: Box<TypeDefinition>,
    },
    Integer,
    List(Box<Type>),
    Map,
    Number,
    String,
    Table,
}

impl AbstractTree for Type {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
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
                    let parameter_type =
                        Type::from_syntax_node(source, parameter_type_node, context)?;

                    parameter_types.push(TypeDefinition::new(parameter_type));
                }

                let return_type_node = node.child(child_count - 1).unwrap();
                let return_type = Type::from_syntax_node(source, return_type_node, context)?;

                Type::Function {
                    parameter_types,
                    return_type: Box::new(TypeDefinition::new(return_type)),
                }
            }
            "int" => Type::Integer,
            "list" => {
                let item_type_node = node.child(1).unwrap();
                let item_type = Type::from_syntax_node(source, item_type_node, context)?;

                Type::List(Box::new(item_type))
            }
            "map" => Type::Map,
            "num" => Type::Number,
            "str" => Type::String,
            "table" => Type::Table,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "any, bool, float, fn, int, list, map, num, str or table",
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
        Ok(TypeDefinition::new(Type::Empty))
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
                write!(f, "fn ")?;

                for parameter_type in parameter_types {
                    write!(f, "{parameter_type} ")?;
                }

                write!(f, "-> {return_type}")
            }
            Type::Integer => write!(f, "int"),
            Type::List(item_type) => write!(f, "list {item_type}"),
            Type::Map => write!(f, "map"),
            Type::Number => write!(f, "num"),
            Type::String => write!(f, "str"),
            Type::Table => write!(f, "table"),
        }
    }
}
