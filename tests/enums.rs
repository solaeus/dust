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
            "Foobar".to_string(),
            "Foo".to_string(),
            Value::none()
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
            "Foobar".to_string(),
            "Bar".to_string(),
            Value::Enum(EnumInstance::new(
                "Fizzbuzz".to_string(),
                "Fizz".to_string(),
                Value::none()
            ))
        )))
    );
}
