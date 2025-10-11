use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character('q')));
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}
