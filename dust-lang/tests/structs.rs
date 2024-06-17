use dust_lang::{
    abstract_tree::{Type, WithPos},
    error::{Error, TypeConflict, ValidationError},
    identifier::Identifier,
    interpret, Value,
};

#[test]
fn simple_structure() {
    assert_eq!(
        interpret(
            "test",
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
            Identifier::new("Foo").with_position((127, 130)),
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
            "test",
            "
                struct Foo {
                    bar : int,
                }

                Foo {
                    bar = 'hiya',
                }
            "
        )
        .unwrap_err()
        .errors(),
        &vec![Error::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::String,
                    expected: Type::Integer
                },
                actual_position: (128, 134).into(),
                expected_position: Some((56, 59).into()),
            },
            position: (96, 153).into()
        }]
    )
}

#[test]
fn nested_structure() {
    assert_eq!(
        interpret(
            "test",
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
            Identifier::new("Foo").with_position((172, 175)),
            vec![(
                Identifier::new("bar"),
                Value::structure(
                    Identifier::new("Bar").with_position((204, 207)),
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
            "test",
            "
                Foo {
                    bar = 42
                }
            "
        )
        .unwrap_err()
        .errors(),
        &vec![Error::Validation {
            error: ValidationError::VariableNotFound {
                identifier: Identifier::new("Foo"),
                position: (17, 20).into()
            },
            position: (17, 69).into()
        }]
    )
}
