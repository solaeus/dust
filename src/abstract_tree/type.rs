use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Identifier, Structure, SyntaxNode, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Any,
    Boolean,
    Collection,
    Custom(Identifier),
    Float,
    Function {
        parameter_types: Vec<Type>,
        return_type: Box<Type>,
    },
    Integer,
    List(Box<Type>),
    Map(Option<Structure>),
    None,
    Number,
    String,
    Range,
    Option(Box<Type>),
}

impl Type {
    pub fn list(item_type: Type) -> Self {
        Type::List(Box::new(item_type))
    }

    pub fn function(parameter_types: Vec<Type>, return_type: Type) -> Self {
        Type::Function {
            parameter_types,
            return_type: Box::new(return_type),
        }
    }

    pub fn option(optional_type: Type) -> Self {
        Type::Option(Box::new(optional_type))
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
            | (Type::Collection, Type::Map(_))
            | (Type::Map(_), Type::Collection)
            | (Type::Collection, Type::String)
            | (Type::String, Type::Collection)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::Map(_), Type::Map(_))
            | (Type::Number, Type::Number)
            | (Type::Number, Type::Integer)
            | (Type::Number, Type::Float)
            | (Type::Integer, Type::Number)
            | (Type::Float, Type::Number)
            | (Type::None, Type::None)
            | (Type::String, Type::String) => true,
            (Type::Custom(left), Type::Custom(right)) => left == right,
            (Type::Option(_), Type::None) => true,
            (Type::Option(left), Type::Option(right)) => {
                if let Type::Any = left.as_ref() {
                    true
                } else if left == right {
                    true
                } else {
                    false
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
        matches!(self, Type::Map(_))
    }
}

impl AbstractTree for Type {
    fn from_syntax(
        node: SyntaxNode,
        _source: &str,
        _context: &Context,
    ) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(_source, "type", node)?;

        let type_node = node.child(0).unwrap();

        let r#type = match type_node.kind() {
            "[" => {
                let item_type_node = node.child(1).unwrap();
                let item_type = Type::from_syntax(item_type_node, _source, _context)?;

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
                        let parameter_type = Type::from_syntax(child, _source, _context)?;

                        parameter_types.push(parameter_type);
                    }
                }

                let final_node = node.child(child_count - 1).unwrap();
                let return_type = if final_node.is_named() {
                    Type::from_syntax(final_node, _source, _context)?
                } else {
                    Type::None
                };

                Type::Function {
                    parameter_types,
                    return_type: Box::new(return_type),
                }
            }
            "int" => Type::Integer,
            "map" => Type::Map(None),
            "num" => Type::Number,
            "none" => Type::None,
            "str" => Type::String,
            "option" => {
                let inner_type_node = node.child(2).unwrap();
                let inner_type = Type::from_syntax(inner_type_node, _source, _context)?;

                Type::Option(Box::new(inner_type))
            }
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "any, bool, float, int, num, str, option, (, [ or {".to_string(),
                    actual: type_node.kind().to_string(),
                    location: type_node.start_position(),
                    relevant_source: _source[type_node.byte_range()].to_string(),
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

            Type::Custom(_) => todo!(),
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
            Type::Map(structure_option) => {
                if let Some(structure) = structure_option {
                    output.push_str(&structure.to_string());
                } else {
                    output.push_str("map");
                }
            }
            Type::None => output.push_str("none"),
            Type::Number => output.push_str("num"),
            Type::String => output.push_str("str"),
            Type::Option(optional_type) => {
                output.push_str("option(");
                optional_type.format(output, indent_level);
                output.push(')');
            }
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
            Type::Custom(identifier) => write!(f, "{identifier}"),
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
            Type::Map(_) => write!(f, "map"),
            Type::Number => write!(f, "num"),
            Type::None => write!(f, "none"),
            Type::String => write!(f, "str"),
            Type::Option(inner_type) => {
                write!(f, "option({})", inner_type)
            }
            Type::Range => todo!(),
        }
    }
}
