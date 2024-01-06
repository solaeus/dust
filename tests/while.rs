use dust_lang::*;

#[test]
fn while_loop() {
    assert_eq!(interpret("while false { 'foo' }"), Ok(Value::Option(None)))
}

#[test]
fn while_loop_iteration_count() {
    assert_eq!(
        interpret("i = 0; while i < 3 { i += 1 }; i"),
        Ok(Value::Integer(3))
    )
}
