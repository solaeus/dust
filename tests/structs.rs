use dust_lang::{abstract_tree::Identifier, error::Error, *};

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
            error: error::ValidationError::TypeNotFound(Identifier::new("Foo")),
            position: (17, 82).into()
        }])
    )
}
