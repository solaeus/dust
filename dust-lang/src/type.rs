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
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::Identifier;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeConflict {
    pub expected: Type,
    pub actual: Type,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
/// Description of a kind of value.
///
/// See the [module documentation](index.html) for more information.
pub enum Type {
    Any,
    Boolean,
    Enum {
        name: Identifier,
        type_parameters: Option<Vec<Type>>,
        variants: Vec<(Identifier, Option<Vec<Type>>)>,
    },
    Float,
    Function {
        type_parameters: Option<Vec<Type>>,
        value_parameters: Option<Vec<(Identifier, Type)>>,
        return_type: Option<Box<Type>>,
    },
    Generic {
        identifier: Identifier,
        concrete_type: Option<Box<Type>>,
    },
    Integer,
    List {
        item_type: Box<Type>,
        length: usize,
    },
    ListOf {
        item_type: Box<Type>,
    },
    Map(BTreeMap<Identifier, Type>),
    Number,
    Range,
    String,
    Struct(StructType),
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
                Type::Function {
                    type_parameters: left_type_parameters,
                    value_parameters: left_value_parameters,
                    return_type: left_return,
                },
                Type::Function {
                    type_parameters: right_type_parameters,
                    value_parameters: right_value_parameters,
                    return_type: right_return,
                },
            ) => {
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
            (Type::Map(left), Type::Map(right)) => {
                if left == right {
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
            Type::Enum { variants, .. } => {
                write!(f, "enum ")?;

                write!(f, " {{")?;

                for (identifier, types) in variants {
                    writeln!(f, "{identifier}")?;

                    if let Some(types) = types {
                        write!(f, "(")?;

                        for r#type in types {
                            write!(f, "{}", r#type)?;
                        }
                    }

                    write!(f, ")")?;
                }

                write!(f, "}}")
            }
            Type::Float => write!(f, "float"),
            Type::Generic { concrete_type, .. } => {
                match concrete_type.clone().map(|r#box| *r#box) {
                    Some(Type::Generic { identifier, .. }) => write!(f, "{identifier}"),
                    Some(concrete_type) => write!(f, "implied to be {concrete_type}"),
                    None => write!(f, "unknown"),
                }
            }
            Type::Integer => write!(f, "int"),
            Type::List { item_type, length } => write!(f, "[{item_type}; {length}]"),
            Type::ListOf { item_type } => write!(f, "list_of({item_type})"),
            Type::Map(map) => {
                write!(f, "{{ ")?;

                for (index, (key, r#type)) in map.iter().enumerate() {
                    write!(f, "{key}: {type}")?;

                    if index != map.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
            Type::Number => write!(f, "num"),
            Type::Range => write!(f, "range"),
            Type::String => write!(f, "str"),
            Type::Function {
                type_parameters,
                value_parameters,
                return_type,
            } => {
                write!(f, "fn ")?;

                if let Some(type_parameters) = type_parameters {
                    write!(f, "<")?;

                    for identifier in type_parameters {
                        write!(f, "{}, ", identifier)?;
                    }

                    write!(f, ">")?;
                }

                write!(f, "(")?;

                if let Some(value_parameters) = value_parameters {
                    for (identifier, r#type) in value_parameters {
                        write!(f, "{identifier}: {type}")?;
                    }
                }

                write!(f, ")")?;

                if let Some(r#type) = return_type {
                    write!(f, " -> {type}")
                } else {
                    Ok(())
                }
            }
            Type::Struct(struct_type) => write!(f, "{struct_type}"),
        }
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
            StructType::Unit { name } => write!(f, "struct {name}"),
            StructType::Tuple { name, fields } => {
                write!(f, "struct {name}(")?;

                for (index, r#type) in fields.iter().enumerate() {
                    write!(f, "{type}")?;

                    if index != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
            StructType::Fields { name, fields } => {
                write!(f, "struct {name} {{")?;

                for (index, (identifier, r#type)) in fields.iter().enumerate() {
                    write!(f, "{identifier}: {type}")?;

                    if index != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_same_types() {
        assert_eq!(Type::Any.check(&Type::Any), Ok(()));
        assert_eq!(Type::Boolean.check(&Type::Boolean), Ok(()));
        assert_eq!(Type::Float.check(&Type::Float), Ok(()));
        assert_eq!(Type::Integer.check(&Type::Integer), Ok(()));
        assert_eq!(
            Type::List {
                item_type: Box::new(Type::Boolean),
                length: 42
            }
            .check(&Type::List {
                item_type: Box::new(Type::Boolean),
                length: 42
            }),
            Ok(())
        );

        let mut map = BTreeMap::new();

        map.insert(Identifier::from("x"), Type::Integer);
        map.insert(Identifier::from("y"), Type::String);
        map.insert(Identifier::from("z"), Type::Map(map.clone()));

        assert_eq!(Type::Map(map.clone()).check(&Type::Map(map)), Ok(()));
        assert_eq!(Type::Range.check(&Type::Range), Ok(()));
        assert_eq!(Type::String.check(&Type::String), Ok(()));
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
            Type::Map(BTreeMap::new()),
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
