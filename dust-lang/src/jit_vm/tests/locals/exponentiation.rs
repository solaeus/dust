use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_byte_exponent() {
    let source = local_cases::LOCAL_BYTE_EXPONENT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(8)));
}

#[test]
fn local_float_exponent() {
    let source = local_cases::LOCAL_FLOAT_EXPONENT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(8.0)));
}

#[test]
fn local_integer_exponent() {
    let source = local_cases::LOCAL_INTEGER_EXPONENT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(8)));
}

#[test]
fn local_mut_byte_exponent() {
    let source = local_cases::LOCAL_MUT_BYTE_EXPONENT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(8)));
}

#[test]
fn local_mut_float_exponent() {
    let source = local_cases::LOCAL_MUT_FLOAT_EXPONENT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(8.0)));
}

#[test]
fn local_mut_integer_exponent() {
    let source = local_cases::LOCAL_MUT_INTEGER_EXPONENT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(8)));
}
