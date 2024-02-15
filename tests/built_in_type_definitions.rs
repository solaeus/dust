use dust_lang::*;

#[test]
fn option() {
    assert_eq!(
        interpret("new Option:None"),
        Ok(Value::Enum(EnumInstance::new(
            "Option".to_string(),
            "None".to_string(),
            Value::none()
        )))
    );
    assert_eq!(
        interpret(
            "
            opt <Option<int>> = new Option:Some(1);

            opt
            "
        ),
        Ok(Value::Enum(EnumInstance::new(
            "Option".to_string(),
            "Some".to_string(),
            Value::Integer(1),
        )))
    );
}

#[test]
fn result() {
    assert_eq!(
        interpret("new Result:Ok(1)"),
        Ok(Value::Enum(EnumInstance::new(
            "Result".to_string(),
            "Ok".to_string(),
            Value::Integer(1)
        )))
    );
    assert_eq!(
        interpret(
            "
            result = new Result:Error('uh-oh!')
            result
            "
        ),
        Ok(Value::Enum(EnumInstance::new(
            "Result".to_string(),
            "Error".to_string(),
            Value::String("uh-oh!".to_string())
        )))
    );
}
