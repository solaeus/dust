use crate::{
    jit_vm::run_main,
    tests::{create_function_with_call_case, local_cases},
    value::Value,
};

#[test]
fn local_byte_division() {
    let source = create_function_with_call_case(local_cases::LOCAL_BYTE_DIVISION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_float_division() {
    let source = create_function_with_call_case(local_cases::LOCAL_FLOAT_DIVISION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_integer_division() {
    let source = create_function_with_call_case(local_cases::LOCAL_INTEGER_DIVISION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_mut_byte_division() {
    let source = create_function_with_call_case(local_cases::LOCAL_MUT_BYTE_DIVISION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_mut_float_division() {
    let source = create_function_with_call_case(local_cases::LOCAL_MUT_FLOAT_DIVISION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_mut_integer_division() {
    let source = create_function_with_call_case(local_cases::LOCAL_MUT_INTEGER_DIVISION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
