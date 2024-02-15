use dust_lang::*;

#[test]
fn simple_enum() {
    let result = interpret(
        "
        enum Foobar {
            Foo,
            Bar,
        }

        new Foobar:Foo
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
            Bar(Fizzbuzz),
        }

        new Foobar:Bar(new Fizzbuzz:Fizz)
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
