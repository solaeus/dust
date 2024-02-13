use dust_lang::{error::ValidationError, *};

#[test]
fn string_as_list() {
    assert_eq!(
        interpret("'foobar' as [str]"),
        Ok(Value::List(List::with_items(vec![
            Value::String("f".to_string()),
            Value::String("o".to_string()),
            Value::String("o".to_string()),
            Value::String("b".to_string()),
            Value::String("a".to_string()),
            Value::String("r".to_string()),
        ])))
    )
}

#[test]
fn string_as_list_conversion_error() {
    assert_eq!(
        interpret("'foobar' as [float]"),
        Err(Error::Validation(ValidationError::ConversionImpossible {
            initial_type: Type::String,
            target_type: Type::List(Box::new(Type::Float))
        }))
    )
}
