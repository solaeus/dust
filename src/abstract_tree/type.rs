use crate::{
    abstract_tree::Identifier,
    context::Context,
    error::{RuntimeError, TypeCheckError, ValidationError},
    Value,
};

use super::AbstractTree;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Any,
    Boolean,
    Custom(Identifier),
    Float,
    Integer,
    List,
    ListOf(Box<Type>),
    ListExact(Vec<Type>),
    Map,
    None,
    Range,
    String,
}

impl Type {
    pub fn check(&self, other: &Type) -> Result<(), TypeCheckError> {
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
            (Type::Custom(left), Type::Custom(right)) => {
                if left == right {
                    Ok(())
                } else {
                    Err(TypeCheckError {
                        actual: other.clone(),
                        expected: self.clone(),
                    })
                }
            }
            (Type::ListOf(left), Type::ListOf(right)) => {
                if let Ok(()) = left.check(right) {
                    Ok(())
                } else {
                    Err(TypeCheckError {
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
            _ => Err(TypeCheckError {
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

    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        Ok(Value::none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_same_types() {
        assert_eq!(Type::Any.check(&Type::Any), Ok(()));
        assert_eq!(Type::Boolean.check(&Type::Boolean), Ok(()));
        assert_eq!(
            Type::Custom(Identifier::new("foo")).check(&Type::Custom(Identifier::new("foo"))),
            Ok(())
        );
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
        let foo = Type::Custom(Identifier::new("foo"));
        let bar = Type::Custom(Identifier::new("bar"));

        assert_eq!(
            foo.check(&bar),
            Err(TypeCheckError {
                actual: bar.clone(),
                expected: foo.clone()
            })
        );
        assert_eq!(
            bar.check(&foo),
            Err(TypeCheckError {
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
                Err(TypeCheckError {
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
