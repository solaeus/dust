use dust_lang::*;

#[test]
fn option() {
    assert_eq!(
        interpret("Option::None"),
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Option"),
            Identifier::new("None"),
            Some(Value::none()),
        )))
    );
    assert_eq!(
        interpret(
            "
            opt <Option<int>> = Option::Some(1);

            opt
            "
        ),
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Option"),
            Identifier::new("None"),
            Some(Value::Integer(1)),
        )))
    );
}

#[test]
fn result() {
    assert_eq!(
        interpret("Result::Ok(1)"),
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Result"),
            Identifier::new("Ok"),
            Some(Value::Integer(1)),
        )))
    );
    assert_eq!(
        interpret(
            "
            result <Result<int, str>> = Result::Error('uh-oh!')
            result
            "
        ),
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Result"),
            Identifier::new("Error"),
            Some(Value::String("uh-oh!".to_string())),
        )))
    );
}
