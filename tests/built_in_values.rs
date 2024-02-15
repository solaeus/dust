use dust_lang::*;

#[test]
fn args() {
    assert!(interpret("args").is_ok_and(|value| value.is_list()));
}

#[test]
fn assert_equal() {
    assert_eq!(
        interpret("assert_equal"),
        Ok(Value::Function(Function::BuiltIn(
            BuiltInFunction::AssertEqual
        )))
    );
    assert_eq!(
        interpret("assert_equal(false, false)"),
        Ok(Value::Enum(EnumInstance::new(
            "Result".to_string(),
            "Ok".to_string(),
            Some(Value::none()),
        )))
    );
    assert_eq!(
        interpret("assert_equal(true, false)"),
        Ok(Value::Enum(EnumInstance::new(
            "Result".to_string(),
            "Error".to_string(),
            Some(Value::none()),
        )))
    );
}
