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
        Ok(Value::Enum(Enum::new("Foo".to_string(), Value::none())))
    );
}
