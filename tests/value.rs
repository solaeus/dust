use dust_lang::*;

#[test]
fn empty() {
    assert_eq!(interpret("x = 9"), Ok(Value::Option(None)));
    assert_eq!(interpret("x = 1 + 1"), Ok(Value::Option(None)));
}

#[test]
fn integer() {
    assert_eq!(interpret("1"), Ok(Value::Integer(1)));
    assert_eq!(interpret("123"), Ok(Value::Integer(123)));
    assert_eq!(interpret("-666"), Ok(Value::Integer(-666)));
}

#[test]
fn integer_overflow() {
    assert_eq!(
        interpret("9223372036854775807 + 1"),
        Ok(Value::Integer(i64::MIN))
    );
    assert_eq!(
        interpret("-9223372036854775808 - 1"),
        Ok(Value::Integer(i64::MAX))
    );
}

#[test]
fn float() {
    assert_eq!(
        interpret("1.7976931348623157e308"),
        Ok(Value::Float(f64::MAX))
    );
    assert_eq!(
        interpret("-1.7976931348623157e308"),
        Ok(Value::Float(f64::MIN))
    );
}

#[test]
fn string() {
    assert_eq!(interpret("\"one\""), Ok(Value::string("one".to_string())));
    assert_eq!(interpret("'one'"), Ok(Value::string("one".to_string())));
    assert_eq!(interpret("`one`"), Ok(Value::string("one".to_string())));
    assert_eq!(interpret("`'one'`"), Ok(Value::string("'one'".to_string())));
    assert_eq!(interpret("'`one`'"), Ok(Value::string("`one`".to_string())));
    assert_eq!(
        interpret("\"'one'\""),
        Ok(Value::string("'one'".to_string()))
    );
}

#[test]
fn list() {
    assert_eq!(
        interpret("[1, 2, 'foobar']"),
        Ok(Value::List(List::with_items(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::string("foobar".to_string()),
        ])))
    );
}

#[test]
fn empty_list() {
    assert_eq!(interpret("[]"), Ok(Value::List(List::new())));
}

#[test]
fn map() {
    let mut map = Map::new();

    map.set("x".to_string(), Value::Integer(1));
    map.set("foo".to_string(), Value::string("bar".to_string()));

    assert_eq!(interpret("{ x = 1, foo = 'bar' }"), Ok(Value::Map(map)));
}

#[test]
fn empty_map() {
    assert_eq!(interpret("{}"), Ok(Value::Map(Map::new())));
}

#[test]
fn map_types() {
    let mut map = Map::new();

    map.set("x".to_string(), Value::Integer(1));
    map.set("foo".to_string(), Value::string("bar".to_string()));

    assert_eq!(
        interpret("{ x <int> = 1, foo <str> = 'bar' }"),
        Ok(Value::Map(map))
    );
}

#[test]
fn map_type_errors() {
    assert_eq!(
        interpret("{ foo <bool> = 'bar' }"),
        Err(Error::Validation(error::ValidationError::TypeCheck {
            expected: Type::Boolean,
            actual: Type::String,
            position: SourcePosition {
                start_byte: 0,
                end_byte: 22,
                start_row: 1,
                start_column: 0,
                end_row: 1,
                end_column: 22
            }
        }))
    );
}

#[test]
fn function() {
    let result = interpret("() <int> { 1 }");
    let value = result.unwrap();
    let function = value.as_function().unwrap();
    let function = if let Function::ContextDefined(function) = function {
        function
    } else {
        panic!("Something is wrong with this test...");
    };

    assert_eq!(&Vec::<Identifier>::with_capacity(0), function.parameters());
    assert_eq!(&Type::Integer, function.return_type());

    let result = interpret("(x <bool>) <bool> { true }");
    let value = result.unwrap();
    let function = value.as_function().unwrap();
    let function = if let Function::ContextDefined(function) = function {
        function
    } else {
        panic!("Something is wrong with this test...");
    };

    assert_eq!(
        &vec![Identifier::new("x".to_string())],
        function.parameters()
    );
    assert_eq!(&Type::Boolean, function.return_type());
}

#[test]
fn option() {
    let result = interpret("x <option(int)> = some(1); x").unwrap();

    assert_eq!(Value::Option(Some(Box::new(Value::Integer(1)))), result);
}

#[test]
fn range() {
    assert_eq!(interpret("0..100"), Ok(Value::range(0, 100)));
}
