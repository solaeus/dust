use dust_lang::*;

#[test]
fn if_expression() {
    assert_eq!(run("if true { 1 }"), Ok(Some(Value::integer(1))));
    assert_eq!(
        run("if 42 == 42 { 1 } else { 2 }"),
        Ok(Some(Value::integer(2)))
    );
}

#[test]
fn less_than() {
    assert_eq!(run("1 < 2"), Ok(Some(Value::boolean(true))));
}

#[test]
fn greater_than() {
    assert_eq!(run("1 > 2"), Ok(Some(Value::boolean(false))));
}

#[test]
fn less_than_or_equal() {
    assert_eq!(run("1 <= 2"), Ok(Some(Value::boolean(true))));
    assert_eq!(run("1 <= 1"), Ok(Some(Value::boolean(true))));
}

#[test]
fn greater_than_or_equal() {
    assert_eq!(run("1 >= 2"), Ok(Some(Value::boolean(false))));
    assert_eq!(run("1 >= 1"), Ok(Some(Value::boolean(true))));
}

#[test]
fn equal() {
    assert_eq!(run("1 == 1"), Ok(Some(Value::boolean(true))));
}

#[test]
fn not_equal() {
    assert_eq!(run("1 != 1"), Ok(Some(Value::boolean(false))));
}

#[test]
fn and() {
    assert_eq!(run("true && true"), Ok(Some(Value::boolean(true))));
    assert_eq!(run("true && false"), Ok(Some(Value::boolean(false))));
    assert_eq!(run("false && true"), Ok(Some(Value::boolean(false))));
    assert_eq!(run("false && false"), Ok(Some(Value::boolean(false))));
}

#[test]
fn or() {
    assert_eq!(run("true || true"), Ok(Some(Value::boolean(true))));
    assert_eq!(run("true || false"), Ok(Some(Value::boolean(true))));
    assert_eq!(run("false || true"), Ok(Some(Value::boolean(true))));
    assert_eq!(run("false || false"), Ok(Some(Value::boolean(false))));
}

#[test]
fn not() {
    assert_eq!(run("!true"), Ok(Some(Value::boolean(false))));
    assert_eq!(run("!false"), Ok(Some(Value::boolean(true))));
}

#[test]
fn long_math() {
    assert_eq!(
        run("1 + 2 * 3 - 4 / 2"),
        Ok(Some(Value::integer(5).into_reference()))
    );
}

#[test]
fn add() {
    assert_eq!(run("1 + 2"), Ok(Some(Value::integer(3))));
}

#[test]
fn subtract() {
    assert_eq!(run("1 - 2"), Ok(Some(Value::integer(-1))));
}

#[test]
fn multiply() {
    assert_eq!(run("2 * 3"), Ok(Some(Value::integer(6))));
}

#[test]
fn divide() {
    assert_eq!(run("6 / 3"), Ok(Some(Value::integer(2))));
}
