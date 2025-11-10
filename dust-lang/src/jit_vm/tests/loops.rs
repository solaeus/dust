use crate::{jit_vm::run_main, tests::loop_cases, value::Value};

#[test]
fn while_loop() {
    let source = loop_cases::WHILE_LOOP.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
