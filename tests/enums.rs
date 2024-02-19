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

#[test]
fn enum_with_argument() {
    env_logger::builder().is_test(true).try_init().unwrap();

    let result = interpret(
        "
        enum FooBar<T> {
            Foo<T>
            Bar
        }
        enum FizzBuzz {
            Fizz
            Buzz
        }

        FooBar::Bar(FizzBuzz::Fizz)
        ",
    );

    assert_eq!(
        result,
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("FooBar"),
            Identifier::new("Bar"),
            Some(Value::Enum(EnumInstance::new(
                Identifier::new("FizzBuzz"),
                Identifier::new("Fizz"),
                Some(Value::none())
            )))
        )))
    );
}
