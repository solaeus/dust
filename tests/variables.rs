use dust_lang::*;

#[test]
fn set_and_get_variable() {
    assert_eq!(interpret("foobar = true; foobar"), Ok(Value::boolean(true)));
}
