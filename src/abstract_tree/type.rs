use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Error, Format, Identifier, Map, Result, Structure, SyntaxNode, Value};

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

    pub fn check(&self, other: &Type) -> Result<()> {
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
            | (Type::String, Type::String) => Ok(()),
            (Type::Custom(left), Type::Custom(right)) => {
                if left == right {
                    Ok(())
                } else {
                    Err(Error::TypeCheck {
                        expected: self.clone(),
                        actual: other.clone(),
                    })
                }
            }
            (Type::Option(left), Type::Option(right)) => {
                if left == right {
                    Ok(())
                } else if let Type::Any = left.as_ref() {
                    Ok(())
                } else if let Type::Any = right.as_ref() {
                    Ok(())
                } else {
                    Err(Error::TypeCheck {
                        expected: self.clone(),
                        actual: other.clone(),
                    })
                }
            }
            (Type::Option(_), Type::None) | (Type::None, Type::Option(_)) => Ok(()),
            (Type::List(self_item_type), Type::List(other_item_type)) => {
                if self_item_type.check(other_item_type).is_err() {
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
                    if self_parameter_type.check(other_parameter_type).is_err() {
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
    fn from_syntax(node: SyntaxNode, _source: &str, _context: &Map) -> Result<Self> {
        Error::expect_syntax_node(_source, "type", node)?;

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
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "any, bool, float, int, num, str, option, (, [ or {".to_string(),
                    actual: type_node.kind().to_string(),
                    location: type_node.start_position(),
                    relevant_source: _source[type_node.byte_range()].to_string(),
                })
            }
        };

        Ok(r#type)
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::None)
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
        }
    }
}