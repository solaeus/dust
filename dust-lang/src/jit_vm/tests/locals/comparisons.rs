use crate::{jit_vm::run_main, tests::local_cases, value::Value};

#[test]
fn local_boolean_greater_than() {
    let source = local_cases::LOCAL_BOOLEAN_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_boolean_less_than() {
    let source = local_cases::LOCAL_BOOLEAN_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_boolean_greater_than_or_equal() {
    let source = local_cases::LOCAL_BOOLEAN_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_boolean_less_than_or_equal() {
    let source = local_cases::LOCAL_BOOLEAN_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_boolean_equal() {
    let source = local_cases::LOCAL_BOOLEAN_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_boolean_not_equal() {
    let source = local_cases::LOCAL_BOOLEAN_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte_greater_than() {
    let source = local_cases::LOCAL_BYTE_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte_less_than() {
    let source = local_cases::LOCAL_BYTE_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte_greater_than_or_equal() {
    let source = local_cases::LOCAL_BYTE_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte_less_than_or_equal() {
    let source = local_cases::LOCAL_BYTE_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte_equal() {
    let source = local_cases::LOCAL_BYTE_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_byte_not_equal() {
    let source = local_cases::LOCAL_BYTE_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_character_greater_than() {
    let source = local_cases::LOCAL_CHARACTER_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_character_less_than() {
    let source = local_cases::LOCAL_CHARACTER_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_character_greater_than_or_equal() {
    let source = local_cases::LOCAL_CHARACTER_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_character_less_than_or_equal() {
    let source = local_cases::LOCAL_CHARACTER_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_character_equal() {
    let source = local_cases::LOCAL_CHARACTER_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_character_not_equal() {
    let source = local_cases::LOCAL_CHARACTER_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_float_greater_than() {
    let source = local_cases::LOCAL_FLOAT_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_float_less_than() {
    let source = local_cases::LOCAL_FLOAT_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_float_greater_than_or_equal() {
    let source = local_cases::LOCAL_FLOAT_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_float_less_than_or_equal() {
    let source = local_cases::LOCAL_FLOAT_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_float_equal() {
    let source = local_cases::LOCAL_FLOAT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_float_not_equal() {
    let source = local_cases::LOCAL_FLOAT_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_integer_greater_than() {
    let source = local_cases::LOCAL_INTEGER_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_integer_less_than() {
    let source = local_cases::LOCAL_INTEGER_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_integer_greater_than_or_equal() {
    let source = local_cases::LOCAL_INTEGER_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_integer_less_than_or_equal() {
    let source = local_cases::LOCAL_INTEGER_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_integer_equal() {
    let source = local_cases::LOCAL_INTEGER_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_integer_not_equal() {
    let source = local_cases::LOCAL_INTEGER_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_string_greater_than() {
    let source = local_cases::LOCAL_STRING_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn local_string_less_than() {
    let source = local_cases::LOCAL_STRING_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn local_string_greater_than_or_equal() {
    let source = local_cases::LOCAL_STRING_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_string_less_than_or_equal() {
    let source = local_cases::LOCAL_STRING_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_string_equal() {
    let source = local_cases::LOCAL_STRING_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_string_not_equal() {
    let source = local_cases::LOCAL_STRING_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}
