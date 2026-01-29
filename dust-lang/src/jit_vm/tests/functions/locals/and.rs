use crate::{
    instruction::OperandType,
    jit_vm::run_main,
    tests::{create_function_with_call_case, local_cases},
    value::Value,
};

#[test]
fn local_boolean_and() {
    let source =
        create_function_with_call_case(local_cases::LOCAL_BOOLEAN_AND, OperandType::BOOLEAN);
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}
