use std::fmt::{self, Display, Formatter};

use clap::error::Result;

use crate::{
    context::Context,
    error::{RuntimeError, TypeConflict, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Action, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Any,
    Argument(Identifier),
    Boolean,
    Float,
    Function {
        parameter_types: Vec<WithPosition<Type>>,
        return_type: Box<WithPosition<Type>>,
    },
    Integer,
    List,
    ListOf(Box<WithPosition<Type>>),
    ListExact(Vec<WithPosition<Type>>),
    Map,
    None,
    Range,
    String,
    Structure {
        name: Identifier,
        fields: Vec<(Identifier, WithPosition<Type>)>,
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
            | (Type::List, Type::List)
            | (Type::List, Type::ListOf(_))
            | (Type::List, Type::ListExact(_))
            | (Type::ListOf(_), Type::List)
            | (Type::ListExact(_), Type::List)
            | (Type::Map, Type::Map)
            | (Type::None, Type::None)
            | (Type::Range, Type::Range)
            | (Type::String, Type::String) => return Ok(()),
            (Type::Argument(left), Type::Argument(right)) => {
                if left == right {
                    return Ok(());
                }
            }
            (Type::ListOf(left), Type::ListOf(right)) => {
                if let Ok(()) = left.item.check(&right.item) {
                    return Ok(());
                }
            }
            (Type::ListOf(list_of), Type::ListExact(list_exact)) => {
                for r#type in list_exact {
                    list_of.item.check(&r#type.item)?;
                }

                return Ok(());
            }
            (Type::ListExact(list_exact), Type::ListOf(list_of)) => {
                for r#type in list_exact {
                    r#type.item.check(&list_of.item)?;
                }

                return Ok(());
            }
            (Type::ListExact(left), Type::ListExact(right)) => {
                for (left, right) in left.iter().zip(right.iter()) {
                    left.item.check(&right.item)?;
                }

                return Ok(());
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
                        if left_field_name != right_field_name || left_type.item != right_type.item
                        {
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
                Type::Function {
                    parameter_types: left_parameters,
                    return_type: left_return,
                },
                Type::Function {
                    parameter_types: right_parameters,
                    return_type: right_return,
                },
            ) => {
                if left_return.item == right_return.item {
                    for (left_parameter, right_parameter) in
                        left_parameters.iter().zip(right_parameters.iter())
                    {
                        if left_parameter.item != right_parameter.item {
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

impl AbstractNode for Type {
    fn expected_type(&self, _: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(self, _context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        Ok(Action::None)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Boolean => write!(f, "bool"),
            Type::Float => write!(f, "float"),
            Type::Integer => write!(f, "int"),
            Type::List => write!(f, "list"),
            Type::ListOf(item_type) => write!(f, "list({})", item_type.item),
            Type::ListExact(item_types) => {
                write!(f, "[")?;

                for (index, item_type) in item_types.into_iter().enumerate() {
                    if index == item_types.len() - 1 {
                        write!(f, "{}", item_type.item)?;
                    } else {
                        write!(f, "{}, ", item_type.item)?;
                    }
                }

                write!(f, "]")
            }
            Type::Map => write!(f, "map"),
            Type::None => write!(f, "none"),
            Type::Range => write!(f, "range"),
            Type::String => write!(f, "str"),
            Type::Function {
                parameter_types,
                return_type,
            } => {
                write!(f, "(")?;

                for r#type in parameter_types {
                    write!(f, "{} ", r#type.item)?;
                }

                write!(f, ") : {}", return_type.item)
            }
            Type::Structure { name, .. } => write!(f, "{name}"),
            Type::Argument(identifier) => write!(f, "{identifier}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::abstract_tree::WithPos;

    use super::*;

    #[test]
    fn check_same_types() {
        assert_eq!(Type::Any.check(&Type::Any), Ok(()));
        assert_eq!(Type::Boolean.check(&Type::Boolean), Ok(()));
        assert_eq!(Type::Float.check(&Type::Float), Ok(()));
        assert_eq!(Type::Integer.check(&Type::Integer), Ok(()));
        assert_eq!(Type::List.check(&Type::List), Ok(()));
        assert_eq!(
            Type::ListOf(Box::new(Type::Integer.with_position((0, 0))))
                .check(&Type::ListOf(Box::new(Type::Integer.with_position((0, 0))))),
            Ok(())
        );

        assert_eq!(
            Type::ListExact(vec![Type::Float.with_position((0, 0))])
                .check(&Type::ListExact(vec![Type::Float.with_position((0, 0))])),
            Ok(())
        );
        assert_eq!(Type::Map.check(&Type::Map), Ok(()));
        assert_eq!(Type::None.check(&Type::None), Ok(()));
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
            Type::Any,
            Type::Boolean,
            Type::Float,
            Type::Integer,
            Type::List,
            Type::ListOf(Box::new(Type::Boolean.with_position((0, 0)))),
            Type::ListExact(vec![Type::Integer.with_position((0, 0))]),
            Type::Map,
            Type::None,
            Type::Range,
            Type::String,
        ];

        for (left, right) in types.iter().zip(types.iter()) {
            if left == right {
                continue;
            }

            assert_eq!(
                left.check(right),
                Err(TypeConflict {
                    actual: right.clone(),
                    expected: left.clone()
                })
            );
        }
    }

    #[test]
    fn check_list_types() {
        let list = Type::List;
        let list_exact = Type::ListExact(vec![
            Type::Integer.with_position((0, 0)),
            Type::Integer.with_position((0, 0)),
        ]);
        let list_of = Type::ListOf(Box::new(Type::Integer.with_position((0, 0))));

        assert_eq!(list.check(&list_exact), Ok(()));
        assert_eq!(list.check(&list_of), Ok(()));
        assert_eq!(list_exact.check(&list), Ok(()));
        assert_eq!(list_exact.check(&list_of), Ok(()));
        assert_eq!(list_of.check(&list), Ok(()));
        assert_eq!(list_of.check(&list_exact), Ok(()));
    }
}
