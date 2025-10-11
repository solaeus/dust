use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_byte_modulo() {
    let source = local_cases::LOCAL_BYTE_MODULO.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(4)));
}

#[test]
fn local_float_modulo() {
    let source = local_cases::LOCAL_FLOAT_MODULO.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(4.0)));
}

#[test]
fn local_integer_modulo() {
    let source = local_cases::LOCAL_INTEGER_MODULO.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(4)));
}

#[test]
fn local_mut_byte_modulo() {
    let source = local_cases::LOCAL_MUT_BYTE_MODULO.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(4)));
}

#[test]
fn local_mut_float_modulo() {
    let source = local_cases::LOCAL_MUT_FLOAT_MODULO.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(4.0)));
}

#[test]
fn local_mut_integer_modulo() {
    let source = local_cases::LOCAL_MUT_INTEGER_MODULO.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(4)));
}
