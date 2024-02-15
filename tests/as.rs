use dust_lang::{
    error::{RuntimeError, ValidationError},
    *,
};

#[test]
fn string_as_string_list() {
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
fn string_as_list_error() {
    assert_eq!(
        interpret("'foobar' as [float]"),
        Err(Error::Validation(ValidationError::ConversionImpossible {
            initial_type: Type::String,
            target_type: Type::List(Box::new(Type::Float))
        }))
    )
}

const JSON: &str = "{ \"x\": 1 }";

#[test]
fn conversion_runtime_error() {
    let json_value = interpret(&format!("json:parse('{JSON}')")).unwrap();

    assert_eq!(
        interpret(&format!("json:parse('{JSON}') as [map]")),
        Err(Error::Runtime(RuntimeError::ConversionImpossible {
            value: json_value,
            target_type: Type::List(Box::new(Type::Map))
        }))
    )
}
