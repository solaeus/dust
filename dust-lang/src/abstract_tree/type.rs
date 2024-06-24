use std::fmt::{self, Display, Formatter};

use clap::error::Result;
use serde::{Deserialize, Serialize};

use crate::{error::TypeConflict, identifier::Identifier};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
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
        type_parameters: Option<Vec<Identifier>>,
        value_parameters: Option<Vec<(Identifier, Type)>>,
        return_type: Option<Box<Type>>,
    },
    Generic {
        identifier: Identifier,
        concrete_type: Option<Box<Type>>,
    },
    Integer,
    List {
        length: usize,
        item_type: Box<Type>,
    },
    ListOf(Box<Type>),
    Map,
    Range,
    String,
    Structure {
        name: Identifier,
        fields: Vec<(Identifier, Type)>,
    },
}

impl Type {
    pub fn check(&self, other: &Type) -> Result<(), TypeConflict> {
        match (self, other) {
            (Type::Any, _)
            | (_, Type::Any)
            | (Type::Boolean, Type::Boolean)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::Map, Type::Map)
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
                    if left.check(&right).is_ok() {
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
            (Type::ListOf(left), Type::ListOf(right)) => {
                if left.check(&right).is_ok() {
                    return Ok(());
                }
            }
            (
                Type::Structure {
                    name: left_name,
                    fields: left_fields,
                },
                Type::Structure {
                    name: right_name,
                    fields: right_fields,
                },
            ) => {
                if left_name == right_name {
                    for ((left_field_name, left_type), (right_field_name, right_type)) in
                        left_fields.iter().zip(right_fields.iter())
                    {
                        if left_field_name != right_field_name || left_type != right_type {
                            return Err(TypeConflict {
                                actual: other.clone(),
                                expected: self.clone(),
                            });
                        }
                    }

                    return Ok(());
                }
            }
            (
                Type::List {
                    length: left_length,
                    item_type: left_type,
                },
                Type::List {
                    length: right_length,
                    item_type: right_type,
                },
            ) => {
                if left_length != right_length || left_type != right_type {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                return Ok(());
            }
            (
                Type::ListOf(left_type),
                Type::List {
                    item_type: right_type,
                    ..
                },
            )
            | (
                Type::List {
                    item_type: right_type,
                    ..
                },
                Type::ListOf(left_type),
            ) => {
                if right_type.check(&left_type).is_err() {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                } else {
                    return Ok(());
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
                if let Some(r#type) = concrete_type {
                    write!(f, "implied to be {}", r#type)
                } else {
                    write!(f, "unknown")
                }
            }
            Type::Integer => write!(f, "int"),
            Type::List { length, item_type } => write!(f, "[{length}; {}]", item_type),
            Type::ListOf(item_type) => write!(f, "list({})", item_type),
            Type::Map => write!(f, "map"),
            Type::Range => write!(f, "range"),
            Type::String => write!(f, "str"),
            Type::Function {
                type_parameters,
                value_parameters,
                return_type,
            } => {
                write!(f, "(")?;

                if let Some(type_parameters) = type_parameters {
                    for identifier in type_parameters {
                        write!(f, "{} ", identifier)?;
                    }

                    write!(f, ")(")?;
                }

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
            Type::Structure { name, .. } => write!(f, "{name}"),
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
                length: 4,
                item_type: Box::new(Type::Boolean),
            }
            .check(&Type::List {
                length: 4,
                item_type: Box::new(Type::Boolean),
            }),
            Ok(())
        );
        assert_eq!(
            Type::ListOf(Box::new(Type::Integer)).check(&Type::ListOf(Box::new(Type::Integer))),
            Ok(())
        );

        assert_eq!(Type::Map.check(&Type::Map), Ok(()));
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
                length: 10,
                item_type: Box::new(Type::Integer),
            },
            Type::ListOf(Box::new(Type::Boolean)),
            Type::Map,
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

    #[test]
    fn check_list_types() {
        let list = Type::List {
            length: 42,
            item_type: Box::new(Type::Integer),
        };
        let list_of = Type::ListOf(Box::new(Type::Integer));

        assert_eq!(list.check(&list_of), Ok(()));
        assert_eq!(list_of.check(&list), Ok(()));
    }
}
