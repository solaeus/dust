use dust_lang::*;

#[test]
fn logic() {
    assert_eq!(
        interpret("test", "1 == 1").unwrap(),
        Some(Value::boolean(true))
    );
    assert_eq!(
        interpret("test", "('42' == '42') && (42 != 0)").unwrap(),
        Some(Value::boolean(true))
    );
}

#[test]
fn math() {
    assert_eq!(interpret("test", "1 + 1").unwrap(), Some(Value::integer(2)));
    assert_eq!(
        interpret("test", "2 * (21 + 19 + 1 * 2) / 2").unwrap(),
        Some(Value::integer(42))
    );
}

#[test]
fn list_index() {
    assert_eq!(
        interpret("test", "foo = [1, 2, 3]; foo[2]").unwrap(),
        Some(Value::integer(3))
    );
}

#[test]
fn map_index() {
    assert_eq!(
        interpret("test", "{ x = 3 }.x").unwrap(),
        Some(Value::integer(3))
    );
    assert_eq!(
        interpret("test", "foo = { x = 3 }; foo.x").unwrap(),
        Some(Value::integer(3))
    );
}
