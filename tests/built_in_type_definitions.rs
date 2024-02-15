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
