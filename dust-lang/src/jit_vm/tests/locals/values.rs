use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character('q')));
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}

#[test]
fn local_function() {
    let source = local_cases::LOCAL_FUNCTION;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
