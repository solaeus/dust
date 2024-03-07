use dust_lang::*;

#[test]
fn logic() {
    assert_eq!(interpret("1 == 1"), Ok(Value::boolean(true)));
    assert_eq!(
        interpret("('42' == '42') && (42 != 0)"),
        Ok(Value::boolean(true))
    );
}
