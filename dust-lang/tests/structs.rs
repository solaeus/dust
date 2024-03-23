use dust_lang::{
    abstract_tree::{Identifier, Type},
    error::{Error, TypeConflict, ValidationError},
    *,
};
#[test]
fn simple_structure() {
    assert_eq!(
        interpret(
            "
                struct Foo {
                    bar : int,
                    baz : str,
                }

                Foo {
                    bar = 42,
                    baz = 'hiya',
                }
            "
        ),
        Ok(Some(Value::structure(
            Identifier::new("Foo"),
            vec![
                (Identifier::new("bar"), Value::integer(42)),
                (Identifier::new("baz"), Value::string("hiya".to_string())),
            ]
        )))
    )
}

#[test]
fn field_type_error() {
    assert_eq!(
        interpret(
            "
                struct Foo {
                    bar : int,
                }

                Foo {
                    bar = 'hiya',
                }
            "
        ),
        Err(vec![Error::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::String,
                    expected: Type::Integer
                },
                actual_position: (128, 134).into(),
                expected_position: (56, 59).into()
            },
            position: (96, 153).into()
        }])
    )
}

#[test]
fn nested_structure() {
    assert_eq!(
        interpret(
            "
                struct Bar {
                    baz : int
                }
                struct Foo {
                    bar : Bar
                }

                Foo {
                    bar = Bar {
                        baz = 42
                    }
                }
            "
        ),
        Ok(Some(Value::structure(
            Identifier::new("Foo"),
            vec![(
                Identifier::new("bar"),
                Value::structure(
                    Identifier::new("Bar"),
                    vec![(Identifier::new("baz"), Value::integer(42))]
                )
            ),]
        )))
    )
}

#[test]
fn undefined_struct() {
    assert_eq!(
        interpret(
            "
                Foo {
                    bar = 42
                }
            "
        ),
        Err(vec![Error::Validation {
            error: error::ValidationError::VariableNotFound(Identifier::new("Foo")),
            position: (17, 69).into()
        }])
    )
}
