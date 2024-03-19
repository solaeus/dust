use std::fmt::{self, Display, Formatter};

use crate::{
    abstract_tree::Identifier,
    context::Context,
    error::{RuntimeError, TypeConflict, ValidationError},
};

use super::{AbstractTree, Action, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Any,
    Boolean,
    Float,
    Function {
        parameter_types: Vec<Type>,
        return_type: Box<Type>,
    },
    Integer,
    List,
    ListOf(Box<Type>),
    ListExact(Vec<Type>),
    Map,
    Named(Identifier),
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
            | (Type::String, Type::String) => Ok(()),
            (Type::ListOf(left), Type::ListOf(right)) => {
                if let Ok(()) = left.check(right) {
                    Ok(())
                } else {
                    Err(TypeConflict {
                        actual: left.as_ref().clone(),
                        expected: right.as_ref().clone(),
                    })
                }
            }
            (Type::ListOf(list_of), Type::ListExact(list_exact)) => {
                for r#type in list_exact {
                    list_of.check(r#type)?;
                }

                Ok(())
            }
            (Type::ListExact(list_exact), Type::ListOf(list_of)) => {
                for r#type in list_exact {
                    r#type.check(&list_of)?;
                }

                Ok(())
            }
            (Type::ListExact(left), Type::ListExact(right)) => {
                for (left, right) in left.iter().zip(right.iter()) {
                    left.check(right)?;
                }

                Ok(())
            }
            _ => Err(TypeConflict {
                actual: other.clone(),
                expected: self.clone(),
            }),
        }
    }
}

impl AbstractTree for Type {
    fn expected_type(&self, _: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(self, _: &Context) -> Result<Action, RuntimeError> {
        Ok(Action::None)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Boolean => write!(f, "boolean"),
            Type::Float => write!(f, "float"),
            Type::Integer => write!(f, "integer"),
            Type::List => write!(f, "list"),
            Type::ListOf(item_type) => write!(f, "list of {item_type}"),
            Type::ListExact(item_types) => {
                write!(f, "[")?;

                for (index, item_type) in item_types.into_iter().enumerate() {
                    if index == item_types.len() - 1 {
                        write!(f, "{item_type}")?;
                    } else {
                        write!(f, "{item_type}, ")?;
                    }
                }

                write!(f, "]")
            }
            Type::Map => write!(f, "map"),
            Type::None => write!(f, "none"),
            Type::Range => write!(f, "range"),
            Type::String => write!(f, "string"),
            Type::Function {
                parameter_types,
                return_type,
            } => {
                write!(f, "(")?;

                for r#type in parameter_types {
                    write!(f, "{} ", r#type)?;
                }

                write!(f, ") : {return_type}")
            }
            Type::Structure { .. } => todo!(),
            Type::Named(name) => write!(f, "{name}"),
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
        assert_eq!(Type::List.check(&Type::List), Ok(()));
        assert_eq!(
            Type::ListOf(Box::new(Type::Integer)).check(&Type::ListOf(Box::new(Type::Integer))),
            Ok(())
        );

        assert_eq!(
            Type::ListExact(vec![Type::Float]).check(&Type::ListExact(vec![Type::Float])),
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
            Type::ListOf(Box::new(Type::Boolean)),
            Type::ListExact(vec![Type::Integer]),
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
        let list_exact = Type::ListExact(vec![Type::Integer, Type::Integer]);
        let list_of = Type::ListOf(Box::new(Type::Integer));

        assert_eq!(list.check(&list_exact), Ok(()));
        assert_eq!(list.check(&list_of), Ok(()));
        assert_eq!(list_exact.check(&list), Ok(()));
        assert_eq!(list_exact.check(&list_of), Ok(()));
        assert_eq!(list_of.check(&list), Ok(()));
        assert_eq!(list_of.check(&list_exact), Ok(()));
    }
}
