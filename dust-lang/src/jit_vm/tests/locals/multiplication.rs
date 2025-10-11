use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_byte_multiplication() {
    let source = local_cases::LOCAL_BYTE_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_float_multiplication() {
    let source = local_cases::LOCAL_FLOAT_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_integer_multiplication() {
    let source = local_cases::LOCAL_INTEGER_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_mut_byte_multiplication() {
    let source = local_cases::LOCAL_MUT_BYTE_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_mut_float_multiplication() {
    let source = local_cases::LOCAL_MUT_FLOAT_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_mut_integer_multiplication() {
    let source = local_cases::LOCAL_MUT_INTEGER_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
