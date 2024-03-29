use dust_lang::{error::RuntimeError, *};

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
            Identifier::new("Result"),
            Identifier::new("Ok"),
            Some(Value::none()),
        )))
    );
    assert_eq!(
        interpret("assert_equal(true, false)"),
        Err(Error::Runtime(RuntimeError::AssertEqualFailed {
            left: true.into(),
            right: false.into()
        }))
    );
}
