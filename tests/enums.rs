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
