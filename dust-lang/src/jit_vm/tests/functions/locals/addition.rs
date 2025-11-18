use crate::{
    jit_vm::run_main,
    tests::{create_function_with_call_case, local_cases},
    value::Value,
};

#[test]
fn local_byte_addition() {
    let source = create_function_with_call_case(local_cases::LOCAL_BYTE_ADDITION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_float_addition() {
    let source = create_function_with_call_case(local_cases::LOCAL_FLOAT_ADDITION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_integer_addition() {
    let source = create_function_with_call_case(local_cases::LOCAL_INTEGER_ADDITION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_string_concatenation() {
    let source = create_function_with_call_case(local_cases::LOCAL_STRING_CONCATENATION, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}

#[test]
fn local_character_concatenation() {
    let source = create_function_with_call_case(local_cases::LOCAL_CHARACTER_CONCATENATION, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("qq")));
}

#[test]
fn local_string_character_concatenation() {
    let source =
        create_function_with_call_case(local_cases::LOCAL_STRING_CHARACTER_CONCATENATION, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("fooq")));
}

#[test]
fn local_character_string_concatenation() {
    let source =
        create_function_with_call_case(local_cases::LOCAL_CHARACTER_STRING_CONCATENATION, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("qfoo")));
}

#[test]
fn local_mut_byte_addition() {
    let source = create_function_with_call_case(local_cases::LOCAL_MUT_BYTE_ADDITION, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(42)));
}

#[test]
fn local_mut_float_addition() {
    let source = create_function_with_call_case(local_cases::LOCAL_MUT_FLOAT_ADDITION, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(42.0)));
}

#[test]
fn local_mut_integer_addition() {
    let source = create_function_with_call_case(local_cases::LOCAL_MUT_INTEGER_ADDITION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn local_mut_string_concatenation() {
    let source = create_function_with_call_case(local_cases::LOCAL_MUT_STRING_CONCATENATION, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("foobar")));
}

#[test]
fn local_mut_string_character_concatenation() {
    let source = create_function_with_call_case(
        local_cases::LOCAL_MUT_STRING_CHARACTER_CONCATENATION,
        "str",
    );
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("fooq")));
}
