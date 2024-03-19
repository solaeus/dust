use dust_lang::{abstract_tree::Identifier, *};

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
