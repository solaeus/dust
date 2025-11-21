use crate::{
    jit_vm::run_main,
    tests::{create_function_with_call_case, list_cases},
    value::Value,
};

#[test]
fn list_boolean() {
    let source = create_function_with_call_case(list_cases::LIST_BOOLEAN, "[bool]");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean_list(vec![true, false, true])));
}

#[test]
fn list_byte() {
    let source = create_function_with_call_case(list_cases::LIST_BYTE, "[byte]");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte_list(vec![0x2A, 0x2B, 0x2C])));
}

#[test]
fn list_character() {
    let source = create_function_with_call_case(list_cases::LIST_CHARACTER, "[char]");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character_list(vec!['a', 'b', 'c'])));
}

#[test]
fn list_float() {
    let source = create_function_with_call_case(list_cases::LIST_FLOAT, "[float]");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float_list(vec![1.0, 2.0, 3.0])));
}

#[test]
fn list_integer() {
    let source = create_function_with_call_case(list_cases::LIST_INTEGER, "[int]");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer_list(vec![1, 2, 3])));
}

#[test]
fn list_string() {
    let source = create_function_with_call_case(list_cases::LIST_STRING, "[str]");
    let result = run_main(source).unwrap();

    assert_eq!(
        result,
        Some(Value::string_list(vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string()
        ]))
    );
}

#[test]
fn list_equal() {
    let source = create_function_with_call_case(list_cases::LIST_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_not_equal() {
    let source = create_function_with_call_case(list_cases::LIST_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_greater_than() {
    let source = create_function_with_call_case(list_cases::LIST_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_less_than() {
    let source = create_function_with_call_case(list_cases::LIST_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_greater_than_or_equal() {
    let source = create_function_with_call_case(list_cases::LIST_GREATER_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_less_than_or_equal() {
    let source = create_function_with_call_case(list_cases::LIST_LESS_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_index_boolean() {
    let source = create_function_with_call_case(list_cases::LIST_INDEX_BOOLEAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_index_byte() {
    let source = create_function_with_call_case(list_cases::LIST_INDEX_BYTE, "byte");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(0x2B)));
}

#[test]
fn list_index_character() {
    let source = create_function_with_call_case(list_cases::LIST_INDEX_CHARACTER, "char");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character('c')));
}

#[test]
fn list_index_float() {
    let source = create_function_with_call_case(list_cases::LIST_INDEX_FLOAT, "float");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(2.0)));
}

#[test]
fn list_index_integer() {
    let source = create_function_with_call_case(list_cases::LIST_INDEX_INTEGER, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(1)));
}

#[test]
fn list_index_string() {
    let source = create_function_with_call_case(list_cases::LIST_INDEX_STRING, "str");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("baz".to_string())));
}

#[test]
fn local_list_boolean() {
    let source = create_function_with_call_case(list_cases::LOCAL_LIST_BOOLEAN, "[bool]");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean_list(vec![true, false, true])));
}

#[test]
fn local_list_equal() {
    let source = create_function_with_call_case(list_cases::LOCAL_LIST_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_not_equal() {
    let source = create_function_with_call_case(list_cases::LOCAL_LIST_NOT_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_greater_than() {
    let source = create_function_with_call_case(list_cases::LOCAL_LIST_GREATER_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_less_than() {
    let source = create_function_with_call_case(list_cases::LOCAL_LIST_LESS_THAN, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_greater_than_or_equal() {
    let source =
        create_function_with_call_case(list_cases::LOCAL_LIST_GREATER_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_less_than_or_equal() {
    let source = create_function_with_call_case(list_cases::LOCAL_LIST_LESS_THAN_OR_EQUAL, "bool");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}
