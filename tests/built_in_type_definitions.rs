use dust_lang::*;

#[test]
fn option() {
    assert_eq!(
        interpret("Option::None"),
        Ok(Value::Enum(EnumInstance::new(
            "Option".to_string(),
            "None".to_string(),
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
            "Option".to_string(),
            "Some".to_string(),
            Some(Value::Integer(1)),
        )))
    );
}

#[test]
fn result() {
    assert_eq!(
        interpret("Result::Ok(1)"),
        Ok(Value::Enum(EnumInstance::new(
            "Result".to_string(),
            "Ok".to_string(),
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
            "Result".to_string(),
            "Error".to_string(),
            Some(Value::String("uh-oh!".to_string())),
        )))
    );
}
