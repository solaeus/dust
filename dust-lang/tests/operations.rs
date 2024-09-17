use dust_lang::*;

#[test]
fn long_math() {
    assert_eq!(run("1 + 2 * 3 - 4 / 2"), Ok(Some(Value::integer(5))));
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
