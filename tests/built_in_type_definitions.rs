use dust_lang::*;

#[test]
fn override_built_ins() {
    assert_eq!(
        interpret(
            "
            enum Option {
                Some<str>
                None
            }
            
            my_option <Option> = Option::Some('foo')
            my_option
            "
        ),
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Option"),
            Identifier::new("Some"),
            Some(Value::String("foo".to_string())),
        )))
    );
}

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
            Option::Some(1)
            "
        ),
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Option"),
            Identifier::new("Some"),
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
            Result::Error('uh-oh!')
            "
        ),
        Ok(Value::Enum(EnumInstance::new(
            Identifier::new("Result"),
            Identifier::new("Error"),
            Some(Value::String("uh-oh!".to_string())),
        )))
    );
}
