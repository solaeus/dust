use crate::{jit_vm::run_main, tests::constant_cases, value::Value};

#[test]
fn boolean() {
    let source = constant_cases::BOOLEAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn byte() {
    let source = constant_cases::BYTE.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn character() {
    let source = constant_cases::CHARACTER.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character('q')));
}

#[test]
fn float() {
    let source = constant_cases::FLOAT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn integer() {
    let source = constant_cases::INTEGER.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn string() {
    let source = constant_cases::STRING.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}

#[test]
fn constant_byte_addition() {
    let source = constant_cases::CONSTANT_BYTE_ADDITION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_addition() {
    let source = constant_cases::CONSTANT_FLOAT_ADDITION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_addition() {
    let source = constant_cases::CONSTANT_INTEGER_ADDITION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_byte_subtraction() {
    let source = constant_cases::CONSTANT_BYTE_SUBTRACTION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_subtraction() {
    let source = constant_cases::CONSTANT_FLOAT_SUBTRACTION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_subtraction() {
    let source = constant_cases::CONSTANT_INTEGER_SUBTRACTION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_byte_multiplication() {
    let source = constant_cases::CONSTANT_BYTE_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_multiplication() {
    let source = constant_cases::CONSTANT_FLOAT_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_multiplication() {
    let source = constant_cases::CONSTANT_INTEGER_MULTIPLICATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_byte_division() {
    let source = constant_cases::CONSTANT_BYTE_DIVISION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_division() {
    let source = constant_cases::CONSTANT_FLOAT_DIVISION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_division() {
    let source = constant_cases::CONSTANT_INTEGER_DIVISION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_integer_negation() {
    let source = constant_cases::CONSTANT_INTEGER_NEGATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(-42)));
}

#[test]
fn constant_float_negation() {
    let source = constant_cases::CONSTANT_FLOAT_NEGATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(-42.0)));
}

#[test]
fn constant_string_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CONCATENATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}

#[test]
fn constant_character_concatenation() {
    let source = constant_cases::CONSTANT_CHARACTER_CONCATENATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("qq")));
}

#[test]
fn constant_string_character_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CHARACTER_CONCATENATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("fooq")));
}

#[test]
fn constant_character_string_concatenation() {
    let source = constant_cases::CONSTANT_CHARACTER_STRING_CONCATENATION.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("qfoo")));
}

#[test]
fn constant_boolean_and() {
    let source = constant_cases::CONSTANT_BOOLEAN_AND.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_boolean_or() {
    let source = constant_cases::CONSTANT_BOOLEAN_OR.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_not() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_boolean_greater_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_less_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_not_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_greater_than() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_less_than() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_equal() {
    let source = constant_cases::CONSTANT_BYTE_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_not_equal() {
    let source = constant_cases::CONSTANT_BYTE_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_greater_than() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_less_than() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_less_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_not_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_greater_than() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_less_than() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_less_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_equal() {
    let source = constant_cases::CONSTANT_FLOAT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_not_equal() {
    let source = constant_cases::CONSTANT_FLOAT_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_greater_than() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_less_than() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_less_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_equal() {
    let source = constant_cases::CONSTANT_INTEGER_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_not_equal() {
    let source = constant_cases::CONSTANT_INTEGER_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_greater_than() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_string_less_than() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_less_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_equal() {
    let source = constant_cases::CONSTANT_STRING_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_not_equal() {
    let source = constant_cases::CONSTANT_STRING_NOT_EQUAL.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}
