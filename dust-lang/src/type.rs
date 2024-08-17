//! Description of a kind of value.
//!
//! Most types are concrete and specific, the exceptions are the Generic and Any types.
//!
//! Generic types are temporary placeholders that describe a type that will be defined later. The
//! interpreter should use the analysis phase to enforce that all Generic types have a concrete
//! type assigned to them before the program is run.
//!
//! The Any type is used in cases where a value's type does not matter. For example, the standard
//! library's "length" function does not care about the type of item in the list, only the list
//! itself. So the input is defined as `[any]`, i.e. `Type::ListOf(Box::new(Type::Any))`.
use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{value::Function, Identifier};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
/// Description of a kind of value.
///
/// See the [module documentation](index.html) for more information.
pub enum Type {
    Any,
    Boolean,
    Byte,
    Character,
    Enum(EnumType),
    Float,
    Function(FunctionType),
    Generic {
        identifier: Identifier,
        concrete_type: Option<Box<Type>>,
    },
    Integer,
    List {
        item_type: Box<Type>,
        length: usize,
    },
    ListEmpty,
    ListOf {
        item_type: Box<Type>,
    },
    Number,
    Range,
    String,
    Struct(StructType),
    Tuple(Vec<Type>),
}

impl Type {
    /// Returns a concrete type, either the type itself or the concrete type of a generic type.
    pub fn concrete_type(&self) -> &Type {
        match self {
            Type::Generic {
                concrete_type: Some(concrete_type),
                ..
            } => concrete_type.concrete_type(),
            _ => self,
        }
    }

    /// Checks that the type is compatible with another type.
    pub fn check(&self, other: &Type) -> Result<(), TypeConflict> {
        match (self.concrete_type(), other.concrete_type()) {
            (Type::Any, _)
            | (_, Type::Any)
            | (Type::Boolean, Type::Boolean)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::Range, Type::Range)
            | (Type::String, Type::String) => return Ok(()),
            (
                Type::Generic {
                    concrete_type: left,
                    ..
                },
                Type::Generic {
                    concrete_type: right,
                    ..
                },
            ) => match (left, right) {
                (Some(left), Some(right)) => {
                    if left.check(right).is_ok() {
                        return Ok(());
                    }
                }
                (None, None) => {
                    return Ok(());
                }
                _ => {}
            },
            (Type::Generic { concrete_type, .. }, other)
            | (other, Type::Generic { concrete_type, .. }) => {
                if let Some(concrete_type) = concrete_type {
                    if other == concrete_type.as_ref() {
                        return Ok(());
                    }
                }
            }
            (Type::Struct(left_struct_type), Type::Struct(right_struct_type)) => {
                if left_struct_type == right_struct_type {
                    return Ok(());
                }
            }
            (
                Type::List {
                    item_type: left_type,
                    length: left_length,
                },
                Type::List {
                    item_type: right_type,
                    length: right_length,
                },
            ) => {
                if left_length != right_length {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                if left_type.check(right_type).is_err() {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                return Ok(());
            }
            (
                Type::ListOf {
                    item_type: left_type,
                },
                Type::ListOf {
                    item_type: right_type,
                },
            ) => {
                if left_type.check(right_type).is_err() {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }
            }
            (
                Type::List {
                    item_type: list_item_type,
                    ..
                },
                Type::ListOf {
                    item_type: list_of_item_type,
                },
            )
            | (
                Type::ListOf {
                    item_type: list_of_item_type,
                },
                Type::List {
                    item_type: list_item_type,
                    ..
                },
            ) => {
                // TODO: This is a hack, remove it.
                if let Type::Any = **list_of_item_type {
                    return Ok(());
                }

                if list_item_type.check(list_of_item_type).is_err() {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }
            }
            (
                Type::Function(FunctionType {
                    name: left_name,
                    type_parameters: left_type_parameters,
                    value_parameters: left_value_parameters,
                    return_type: left_return,
                }),
                Type::Function(FunctionType {
                    name: right_name,
                    type_parameters: right_type_parameters,
                    value_parameters: right_value_parameters,
                    return_type: right_return,
                }),
            ) => {
                if left_name != right_name {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                if left_return == right_return {
                    for (left_parameter, right_parameter) in left_type_parameters
                        .iter()
                        .zip(right_type_parameters.iter())
                    {
                        if left_parameter != right_parameter {
                            return Err(TypeConflict {
                                actual: other.clone(),
                                expected: self.clone(),
                            });
                        }
                    }

                    for (left_parameter, right_parameter) in left_value_parameters
                        .iter()
                        .zip(right_value_parameters.iter())
                    {
                        if left_parameter != right_parameter {
                            return Err(TypeConflict {
                                actual: other.clone(),
                                expected: self.clone(),
                            });
                        }
                    }

                    return Ok(());
                }
            }
            (Type::Number, Type::Number | Type::Integer | Type::Float)
            | (Type::Integer | Type::Float, Type::Number) => {
                return Ok(());
            }
            _ => {}
        }

        Err(TypeConflict {
            actual: other.clone(),
            expected: self.clone(),
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Boolean => write!(f, "bool"),
            Type::Byte => write!(f, "byte"),
            Type::Character => write!(f, "char"),
            Type::Enum(enum_type) => write!(f, "{enum_type}"),
            Type::Float => write!(f, "float"),
            Type::Function(function_type) => write!(f, "{function_type}"),
            Type::Generic { concrete_type, .. } => {
                match concrete_type.clone().map(|r#box| *r#box) {
                    Some(Type::Generic { identifier, .. }) => write!(f, "{identifier}"),
                    Some(concrete_type) => write!(f, "implied to be {concrete_type}"),
                    None => write!(f, "unknown"),
                }
            }
            Type::Integer => write!(f, "int"),
            Type::List { item_type, length } => write!(f, "[{item_type}; {length}]"),
            Type::ListEmpty => write!(f, "[]"),
            Type::ListOf { item_type } => write!(f, "[{item_type}]"),
            Type::Number => write!(f, "num"),
            Type::Range => write!(f, "range"),
            Type::String => write!(f, "str"),
            Type::Struct(struct_type) => write!(f, "{struct_type}"),
            Type::Tuple(fields) => {
                write!(f, "(")?;

                for (index, r#type) in fields.iter().enumerate() {
                    write!(f, "{type}")?;

                    if index != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionType {
    pub name: Identifier,
    pub type_parameters: Option<Vec<Type>>,
    pub value_parameters: Option<Vec<(Identifier, Type)>>,
    pub return_type: Option<Box<Type>>,
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn ")?;

        if let Some(type_parameters) = &self.type_parameters {
            write!(f, "<")?;

            for (index, type_parameter) in type_parameters.iter().enumerate() {
                write!(f, "{type_parameter}")?;

                if index != type_parameters.len() - 1 {
                    write!(f, ", ")?;
                }
            }

            write!(f, ">")?;
        }

        write!(f, "(")?;

        if let Some(value_parameters) = &self.value_parameters {
            for (index, (identifier, r#type)) in value_parameters.iter().enumerate() {
                write!(f, "{identifier}: {type}")?;

                if index != value_parameters.len() - 1 {
                    write!(f, ", ")?;
                }
            }
        }

        write!(f, ")")?;

        if let Some(return_type) = &self.return_type {
            write!(f, " -> {return_type}")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StructType {
    Unit {
        name: Identifier,
    },
    Tuple {
        name: Identifier,
        fields: Vec<Type>,
    },
    Fields {
        name: Identifier,
        fields: Vec<(Identifier, Type)>,
    },
}

impl Display for StructType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            StructType::Unit { .. } => write!(f, "()"),
            StructType::Tuple { fields, .. } => {
                write!(f, "(")?;

                for (index, r#type) in fields.iter().enumerate() {
                    write!(f, "{type}")?;

                    if index != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
            StructType::Fields {
                name: identifier,
                fields,
                ..
            } => {
                write!(f, "{identifier} {{ ")?;

                for (index, (identifier, r#type)) in fields.iter().enumerate() {
                    write!(f, "{identifier}: {type}")?;

                    if index != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumType {
    name: Identifier,
    variants: Vec<StructType>,
}

impl Display for EnumType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeConflict {
    pub expected: Type,
    pub actual: Type,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_type_any() {
        let foo = Type::Any;
        let bar = Type::Any;

        foo.check(&bar).unwrap();
    }

    #[test]
    fn check_type_boolean() {
        let foo = Type::Boolean;
        let bar = Type::Boolean;

        foo.check(&bar).unwrap();
    }

    #[test]
    fn check_type_byte() {
        let foo = Type::Byte;
        let bar = Type::Byte;

        foo.check(&bar).unwrap();
    }

    #[test]
    fn check_type_character() {
        let foo = Type::Character;
        let bar = Type::Character;

        foo.check(&bar).unwrap();
    }

    #[test]
    fn errors() {
        let foo = Type::Integer;
        let bar = Type::String;

        assert_eq!(
            foo.check(&bar),
            Err(TypeConflict {
                actual: bar.clone(),
                expected: foo.clone()
            })
        );
        assert_eq!(
            bar.check(&foo),
            Err(TypeConflict {
                actual: foo.clone(),
                expected: bar.clone()
            })
        );

        let types = [
            Type::Boolean,
            Type::Float,
            Type::Integer,
            Type::List {
                item_type: Box::new(Type::Integer),
                length: 42,
            },
            Type::Range,
            Type::String,
        ];

        for left in types.clone() {
            for right in types.clone() {
                if left == right {
                    continue;
                }

                assert_eq!(
                    left.check(&right),
                    Err(TypeConflict {
                        actual: right.clone(),
                        expected: left.clone()
                    })
                );
            }
        }
    }
}
