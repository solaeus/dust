use crate::{
    instruction::ByteType,
    jit_vm::run_main,
    tests::{create_function_with_call_case, local_cases},
    value::Value,
};

#[test]
fn local_byte_multiplication() {
    let source =
        create_function_with_call_case(local_cases::LOCAL_BYTE_MULTIPLICATION, ByteType::BYTE);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_float_multiplication() {
    let source =
        create_function_with_call_case(local_cases::LOCAL_FLOAT_MULTIPLICATION, ByteType::FLOAT);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_integer_multiplication() {
    let source = create_function_with_call_case(
        local_cases::LOCAL_INTEGER_MULTIPLICATION,
        ByteType::INTEGER,
    );
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_mut_byte_multiplication() {
    let source = create_function_with_call_case(
        local_cases::LOCAL_MUT_BYTE_MULTIPLICATION,
        ByteType::BYTE,
    );
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_mut_float_multiplication() {
    let source = create_function_with_call_case(
        local_cases::LOCAL_MUT_FLOAT_MULTIPLICATION,
        ByteType::FLOAT,
    );
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_mut_integer_multiplication() {
    let source = create_function_with_call_case(
        local_cases::LOCAL_MUT_INTEGER_MULTIPLICATION,
        ByteType::INTEGER,
    );
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
