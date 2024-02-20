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

#[test]
fn ends_with() {
    let result = interpret("str:ends_with('abc', 'c')");

    assert_eq!(result, Ok(Value::Boolean(true)));

    let result = interpret("str:ends_with('abc', 'b')");

    assert_eq!(result, Ok(Value::Boolean(false)));
}

#[test]
fn find() {
    let result = interpret("str:find('abc', 'a')");

    assert_eq!(result, Ok(Value::some(Value::Integer(0))));

    let result = interpret("str:find('foobar', 'b')");

    assert_eq!(result, Ok(Value::some(Value::Integer(3))));
}

#[test]
fn insert() {
    assert_eq!(
        interpret("str:insert('ac', 1, 'b')"),
        Ok(Value::string("abc"))
    );
    assert_eq!(
        interpret("str:insert('foo', 3, 'bar')"),
        Ok(Value::string("foobar"))
    );
}
