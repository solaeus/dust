use dust_lang::{identifier::Identifier, *};

#[test]
fn simple_enum() {
    assert_eq!(
        interpret(
            "test",
            "
            type FooBar = enum {
                Foo,
                Bar,
            }

            FooBar::Foo
            "
        ),
        Ok(Some(Value::enum_instance(
            Identifier::new("FooBar"),
            Identifier::new("Foo"),
            None
        )))
    );
}
