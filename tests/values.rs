use std::collections::BTreeMap;

use dust_lang::{
    abstract_tree::{Identifier, Type},
    error::{Error, TypeConflict, ValidationError},
    *,
};

#[test]
fn none() {
    assert_eq!(interpret("x = 9"), Ok(None));
    assert_eq!(interpret("x = 1 + 1"), Ok(None));
}

#[test]
fn integer() {
    assert_eq!(interpret("1"), Ok(Some(Value::integer(1))));
    assert_eq!(interpret("123"), Ok(Some(Value::integer(123))));
    assert_eq!(interpret("-666"), Ok(Some(Value::integer(-666))));
}

#[test]
fn integer_saturation() {
    assert_eq!(
        interpret("9223372036854775807 + 1"),
        Ok(Some(Value::integer(i64::MAX)))
    );
    assert_eq!(
        interpret("-9223372036854775808 - 1"),
        Ok(Some(Value::integer(i64::MIN)))
    );
}

#[test]
fn float() {
    assert_eq!(
        interpret("1.7976931348623157e308"),
        Ok(Some(Value::float(f64::MAX)))
    );
    assert_eq!(
        interpret("-1.7976931348623157e308"),
        Ok(Some(Value::float(f64::MIN)))
    );
}

#[test]
fn float_saturation() {
    assert_eq!(
        interpret("1.7976931348623157e308 + 1"),
        Ok(Some(Value::float(f64::MAX)))
    );
    assert_eq!(
        interpret("-1.7976931348623157e308 - 1"),
        Ok(Some(Value::float(f64::MIN)))
    );
}

#[test]
fn string() {
    assert_eq!(
        interpret("\"one\""),
        Ok(Some(Value::string("one".to_string())))
    );
    assert_eq!(
        interpret("'one'"),
        Ok(Some(Value::string("one".to_string())))
    );
    assert_eq!(
        interpret("`one`"),
        Ok(Some(Value::string("one".to_string())))
    );
    assert_eq!(
        interpret("`'one'`"),
        Ok(Some(Value::string("'one'".to_string())))
    );
    assert_eq!(
        interpret("'`one`'"),
        Ok(Some(Value::string("`one`".to_string())))
    );
    assert_eq!(
        interpret("\"'one'\""),
        Ok(Some(Value::string("'one'".to_string())))
    );
}

#[test]
fn list() {
    assert_eq!(
        interpret("[1, 2, 'foobar']"),
        Ok(Some(Value::list(vec![
            Value::integer(1),
            Value::integer(2),
            Value::string("foobar".to_string()),
        ])))
    );
}

#[test]
fn empty_list() {
    assert_eq!(interpret("[]"), Ok(Some(Value::list(Vec::new()))));
}

#[test]
fn map() {
    let mut map = BTreeMap::new();

    map.insert(Identifier::new("x"), Value::integer(1));
    map.insert(Identifier::new("foo"), Value::string("bar".to_string()));

    assert_eq!(
        interpret("{ x = 1, foo = 'bar' }"),
        Ok(Some(Value::map(map)))
    );
}

#[test]
fn empty_map() {
    assert_eq!(interpret("{}"), Ok(Some(Value::map(BTreeMap::new()))));
}

#[test]
fn map_types() {
    let mut map = BTreeMap::new();

    map.insert(Identifier::new("x"), Value::integer(1));
    map.insert(Identifier::new("foo"), Value::string("bar".to_string()));

    assert_eq!(
        interpret("{ x : int = 1, foo : str = 'bar' }"),
        Ok(Some(Value::map(map)))
    );
}

#[test]
fn map_type_errors() {
    assert_eq!(
        interpret("{ foo : bool = 'bar' }"),
        Err(vec![Error::Validation {
            error: ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::String,
                    expected: Type::Boolean
                },
                actual_position: (0, 0),
                expected_position: (0, 0),
            },
            position: (0, 22)
        }])
    );
}

#[test]
fn range() {
    assert_eq!(interpret("0..100"), Ok(Some(Value::range(0..100))));
}
