use crate::{
    instruction::OperandType,
    jit_vm::run_main,
    tests::{create_function_with_call_case, local_cases},
    value::Value,
};

#[test]
fn local_boolean() {
    let source = create_function_with_call_case(local_cases::LOCAL_BOOLEAN, OperandType::BOOLEAN);
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte() {
    let source = create_function_with_call_case(local_cases::LOCAL_BYTE, OperandType::BYTE);
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_character() {
    let source = create_function_with_call_case(local_cases::LOCAL_CHARACTER, OperandType::CHARACTER);
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character('q')));
}

#[test]
fn local_float() {
    let source = create_function_with_call_case(local_cases::LOCAL_FLOAT, OperandType::FLOAT);
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_integer() {
    let source = create_function_with_call_case(local_cases::LOCAL_INTEGER, OperandType::INTEGER);
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_string() {
    let source = create_function_with_call_case(local_cases::LOCAL_STRING, OperandType::STRING);
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}
