use dust_lang::*;

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
