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

#[test]
fn big_enum() {
    assert_eq!(
        interpret(
            "test",
            "
            type FooBarBaz = enum |T, U, V| {
                Foo(T),
                Bar(U),
                Baz(V),
            }

            FooBarBaz::Baz(42.0)
            "
        ),
        Ok(Some(Value::enum_instance(
            Identifier::new("FooBarBaz"),
            Identifier::new("Baz"),
            Some(vec![Value::float(42.0)]),
        )))
    );
}
