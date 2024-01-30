use dust_lang::{interpret, List, Value};

#[test]
fn as_bytes() {
    let result = interpret("str:as_bytes('123')");

    assert_eq!(
        result,
        Ok(Value::List(List::with_items(vec![
            Value::Integer(49),
            Value::Integer(50),
            Value::Integer(51),
        ])))
    );
}
