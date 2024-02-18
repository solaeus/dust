use dust_lang::*;

#[test]
fn simple_enum() {
    let result = interpret(
        "
        enum Foobar {
            Foo,
            Bar,
        }

        Foobar::Foo
        ",
    );

    assert_eq!(
        result,
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Foobar"),
            Identifier::new("Foo"),
            Some(Value::none())
        )))
    );
}

#[test]
fn nested_enum() {
    let result = interpret(
        "
        enum Fizzbuzz {
            Fizz,
            Buzz,
        }
        enum Foobar {
            Foo,
            Bar<Fizzbuzz>,
        }

        Foobar::Bar(Fizzbuzz::Fizz)
        ",
    );

    assert_eq!(
        result,
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Foobar"),
            Identifier::new("Bar"),
            Some(Value::Enum(EnumInstance::new(
                Identifier::new("Fizzbuzz"),
                Identifier::new("Fizz"),
                Some(Value::none())
            )))
        )))
    );
}
