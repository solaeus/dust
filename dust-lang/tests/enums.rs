use dust_lang::{identifier::Identifier, *};

#[test]
fn simple_enum() {
    assert_eq!(
        interpret(
            "test",
            "
            enum FooBar {
                Foo,
                Bar,
            }

            FooBar::Foo
            "
        ),
        Ok(Some(Value::enum_instance(
            Identifier::new("FooBar"),
            None,
            Identifier::new("Foo"),
            None
        )))
    );
}

#[test]
fn big_enum() {
    assert_eq!(
        interpret(
            "test",
            "
            enum FooBarBaz <T, U, V> {
                Foo(T),
                Bar(U),
                Baz(V),
            }

            FooBarBaz::<int, str, float>::Baz(42.0)
            "
        ),
        Ok(Some(Value::enum_instance(
            Identifier::new("FooBarBaz"),
            Some(vec![Type::Integer, Type::String, Type::Float]),
            Identifier::new("Baz"),
            Some(Value::float(42.0)),
        )))
    );
}
