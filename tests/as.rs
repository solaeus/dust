use dust_lang::{
    error::{RuntimeError, ValidationError},
    *,
};

#[test]
fn string_as_string_list() {
    assert_eq!(
        interpret("'foobar' as [str]"),
        Ok(Value::List(List::with_items(vec![
            Value::string("f"),
            Value::string("o"),
            Value::string("o"),
            Value::string("b"),
            Value::string("a"),
            Value::string("r"),
        ])))
    )
}

#[test]
fn string_as_list_error() {
    assert_eq!(
        interpret("'foobar' as [float]"),
        Err(Error::Validation(ValidationError::ConversionImpossible {
            initial_type: Type::String,
            target_type: Type::ListOf(Box::new(Type::Float))
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
            from: json_value.r#type().unwrap(),
            to: Type::ListOf(Box::new(Type::Map(None))),
            position: SourcePosition {
                start_byte: 0,
                end_byte: 33,
                start_row: 1,
                start_column: 0,
                end_row: 1,
                end_column: 33,
            }
        }))
    )
}
