use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    built_in_types::BuiltInType,
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Identifier, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Any,
    Boolean,
    Collection,
    Custom {
        name: Identifier,
        argument: Option<Box<Type>>,
    },
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
    Range,
    None,
}

impl Type {
    pub fn custom(name: Identifier, argument: Option<Type>) -> Self {
        Type::Custom {
            name,
            argument: argument.map(|r#type| Box::new(r#type)),
        }
    }

    pub fn option(inner_type: Option<Type>) -> Self {
        BuiltInType::Option(inner_type).get().clone()
    }

    pub fn list(item_type: Type) -> Self {
        Type::List(Box::new(item_type))
    }

    pub fn function(parameter_types: Vec<Type>, return_type: Type) -> Self {
        Type::Function {
            parameter_types,
            return_type: Box::new(return_type),
        }
    }

    /// Returns a boolean indicating whether is type is accepting of the other.
    ///
    /// The types do not need to match exactly. For example, the Any variant matches all of the
    /// others and the Number variant accepts Number, Integer and Float.
    pub fn accepts(&self, other: &Type) -> bool {
        log::info!("Checking type {self} against {other}.");

        match (self, other) {
            (Type::Any, _)
            | (_, Type::Any)
            | (Type::Boolean, Type::Boolean)
            | (Type::Collection, Type::Collection)
            | (Type::Collection, Type::List(_))
            | (Type::List(_), Type::Collection)
            | (Type::Collection, Type::Map)
            | (Type::Map, Type::Collection)
            | (Type::Collection, Type::String)
            | (Type::String, Type::Collection)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::Map, Type::Map)
            | (Type::Number, Type::Number)
            | (Type::Number, Type::Integer)
            | (Type::Number, Type::Float)
            | (Type::Integer, Type::Number)
            | (Type::Float, Type::Number)
            | (Type::String, Type::String)
            | (Type::None, Type::None) => true,
            (
                Type::Custom {
                    name: left_name,
                    argument: left_argument,
                },
                Type::Custom {
                    name: right_name,
                    argument: right_argument,
                },
            ) => {
                if left_name != right_name {
                    false
                } else {
                    left_argument == right_argument
                }
            }
            (Type::List(self_item_type), Type::List(other_item_type)) => {
                self_item_type.accepts(&other_item_type)
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
                    if self_parameter_type == other_parameter_type {
                        return false;
                    }
                }

                self_return_type == other_return_type
            }
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function { .. })
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Type::List(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Type::Map)
    }
}

impl AbstractTree for Type {
    fn from_syntax(
        node: SyntaxNode,
        _source: &str,
        context: &Context,
    ) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("type", node)?;

        let type_node = node.child(0).unwrap();

        let r#type = match type_node.kind() {
            "identifier" => {
                let name = Identifier::from_syntax(type_node, _source, context)?;
                let argument = if let Some(child) = node.child(2) {
                    Some(Type::from_syntax(child, _source, context)?)
                } else {
                    None
                };

                Type::custom(name, argument)
            }
            "[" => {
                let item_type_node = node.child(1).unwrap();
                let item_type = Type::from_syntax(item_type_node, _source, context)?;

                Type::List(Box::new(item_type))
            }
            "any" => Type::Any,
            "bool" => Type::Boolean,
            "collection" => Type::Collection,
            "float" => Type::Float,
            "(" => {
                let child_count = node.child_count();
                let mut parameter_types = Vec::new();

                for index in 1..child_count - 2 {
                    let child = node.child(index).unwrap();

                    if child.is_named() {
                        let parameter_type = Type::from_syntax(child, _source, context)?;

                        parameter_types.push(parameter_type);
                    }
                }

                let final_node = node.child(child_count - 1).unwrap();
                let return_type = if final_node.is_named() {
                    Type::from_syntax(final_node, _source, context)?
                } else {
                    Type::option(None)
                };

                Type::Function {
                    parameter_types,
                    return_type: Box::new(return_type),
                }
            }
            "int" => Type::Integer,
            "map" => Type::Map,
            "num" => Type::Number,
            "none" => Type::None,
            "str" => Type::String,
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "any, bool, float, int, num, str, custom type, (, [ or {".to_string(),
                    actual: type_node.kind().to_string(),
                    position: node.range().into(),
                })
            }
        };

        Ok(r#type)
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        Ok(Value::none())
    }
}

impl Format for Type {
    fn format(&self, output: &mut String, indent_level: u8) {
        match self {
            Type::Any => output.push_str("any"),
            Type::Boolean => output.push_str("bool"),
            Type::Collection => output.push_str("collection"),
            Type::Custom {
                name: _,
                argument: _,
            } => todo!(),
            Type::Float => output.push_str("float"),
            Type::Function {
                parameter_types,
                return_type,
            } => {
                output.push('(');

                for (index, parameter_type) in parameter_types.iter().enumerate() {
                    parameter_type.format(output, indent_level);

                    if index != parameter_types.len() - 1 {
                        output.push(' ');
                    }
                }

                output.push_str(") -> ");
                return_type.format(output, indent_level);
            }
            Type::Integer => output.push_str("int"),
            Type::List(item_type) => {
                output.push('[');
                item_type.format(output, indent_level);
                output.push(']');
            }
            Type::Map => {
                output.push_str("map");
            }
            Type::None => output.push_str("Option::None"),
            Type::Number => output.push_str("num"),
            Type::String => output.push_str("str"),
            Type::Range => todo!(),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Boolean => write!(f, "bool"),
            Type::Collection => write!(f, "collection"),
            Type::Custom { name, argument } => {
                if let Some(argument) = argument {
                    write!(f, "{name}<{argument}>")
                } else {
                    write!(f, "{name}")
                }
            }
            Type::Float => write!(f, "float"),
            Type::Function {
                parameter_types,
                return_type,
            } => {
                write!(f, "(")?;

                for (index, parameter_type) in parameter_types.iter().enumerate() {
                    write!(f, "{parameter_type}")?;

                    if index != parameter_types.len() - 1 {
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
            Type::None => write!(f, "none"),
            Type::String => write!(f, "str"),
            Type::Range => todo!(),
        }
    }
}
