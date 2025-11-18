use crate::{
    jit_vm::run_main,
    tests::{constant_cases, create_function_with_call_case},
    value::Value,
};

#[test]
fn boolean() {
    let source = create_function_with_call_case(constant_cases::BOOLEAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn byte() {
    let source = create_function_with_call_case(constant_cases::BYTE, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn character() {
    let source = create_function_with_call_case(constant_cases::CHARACTER, "char");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character('q')));
}

#[test]
fn float() {
    let source = create_function_with_call_case(constant_cases::FLOAT, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn integer() {
    let source = create_function_with_call_case(constant_cases::INTEGER, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn string() {
    let source = create_function_with_call_case(constant_cases::STRING, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}

#[test]
fn constant_byte_addition() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_ADDITION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_addition() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_FLOAT_ADDITION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_addition() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_INTEGER_ADDITION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_byte_subtraction() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_SUBTRACTION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_subtraction() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_FLOAT_SUBTRACTION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_subtraction() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_INTEGER_SUBTRACTION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_byte_multiplication() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_BYTE_MULTIPLICATION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_multiplication() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_FLOAT_MULTIPLICATION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_multiplication() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_INTEGER_MULTIPLICATION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_byte_division() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_DIVISION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn constant_float_division() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_FLOAT_DIVISION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn constant_integer_division() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_INTEGER_DIVISION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn constant_byte_modulo() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_MODULO, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(4)));
}

#[test]
fn constant_float_modulo() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_FLOAT_MODULO, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(4.0)));
}

#[test]
fn constant_integer_modulo() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_INTEGER_MODULO, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(4)));
}

#[test]
fn constant_integer_negation() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_INTEGER_NEGATION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(-42)));
}

#[test]
fn constant_float_negation() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_FLOAT_NEGATION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(-42.0)));
}

#[test]
fn constant_string_concatenation() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_STRING_CONCATENATION, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}

#[test]
fn constant_character_concatentation() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_CHARACTER_CONCATENATION, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("qq")));
}

#[test]
fn constant_string_character_concatenation() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_STRING_CHARACTER_CONCATENATION,
        "str",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("fooq")));
}

#[test]
fn constant_character_string_concatenation() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_CHARACTER_STRING_CONCATENATION,
        "str",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("qfoo")));
}

#[test]
fn constant_boolean_and() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_AND, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_boolean_or() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_OR, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_not() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_NOT, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_boolean_greater_than() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_less_than() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_greater_than_or_equal() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL,
        "bool",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_less_than_or_equal() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_boolean_not_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BOOLEAN_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_greater_than() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_less_than() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_greater_than_or_equal() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_BYTE_GREATER_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_less_than_or_equal() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_byte_not_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_BYTE_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_greater_than() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_CHARACTER_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_less_than() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_CHARACTER_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_greater_than_or_equal() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL,
        "bool",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_less_than_or_equal() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL,
        "bool",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_CHARACTER_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_character_not_equal() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_CHARACTER_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_greater_than() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_FLOAT_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_less_than() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_FLOAT_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_greater_than_or_equal() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL,
        "bool",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_less_than_or_equal() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_FLOAT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_float_not_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_FLOAT_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_greater_than() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_INTEGER_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_less_than() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_INTEGER_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_greater_than_or_equal() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL,
        "bool",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_less_than_or_equal() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_INTEGER_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_integer_not_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_INTEGER_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_greater_than() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_STRING_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_string_less_than() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_STRING_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(false)));
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = create_function_with_call_case(
        constant_cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL,
        "bool",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_less_than_or_equal() {
    let source =
        create_function_with_call_case(constant_cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_STRING_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn constant_string_not_equal() {
    let source = create_function_with_call_case(constant_cases::CONSTANT_STRING_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}
