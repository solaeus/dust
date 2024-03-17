use dust_lang::*;

#[test]
fn logic() {
    assert_eq!(interpret("1 == 1").unwrap(), Some(Value::boolean(true)));
    assert_eq!(
        interpret("('42' == '42') && (42 != 0)").unwrap(),
        Some(Value::boolean(true))
    );
}

#[test]
fn math() {
    assert_eq!(interpret("1 + 1").unwrap(), Some(Value::integer(2)));
    assert_eq!(
        interpret("2 * (21 + 19 + 1 * 2) / 2").unwrap(),
        Some(Value::integer(42))
    );
}

#[test]
fn list_index() {
    assert_eq!(
        interpret("foo = [1, 2, 3]; foo.2").unwrap(),
        Some(Value::integer(3))
    );
}
