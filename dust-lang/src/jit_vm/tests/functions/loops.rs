use crate::{
    jit_vm::run_main,
    tests::{create_function_with_call_case, loop_cases},
    value::Value,
};

#[test]
fn while_loop() {
    let source = create_function_with_call_case(loop_cases::WHILE_LOOP, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
