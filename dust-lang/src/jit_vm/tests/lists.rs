use crate::{jit_vm::run_main, tests::list_cases, value::Value};

#[test]
fn list_boolean() {
    let source = list_cases::LIST_BOOLEAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean_list(vec![true, false, true])));
}

#[test]
fn list_byte() {
    let source = list_cases::LIST_BYTE;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte_list(vec![0x2A, 0x2B, 0x2C])));
}

#[test]
fn list_character() {
    let source = list_cases::LIST_CHARACTER;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character_list(vec!['a', 'b', 'c'])));
}

#[test]
fn list_float() {
    let source = list_cases::LIST_FLOAT;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float_list(vec![1.0, 2.0, 3.0])));
}

#[test]
fn list_integer() {
    let source = list_cases::LIST_INTEGER;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer_list(vec![1, 2, 3])));
}

#[test]
fn list_string() {
    let source = list_cases::LIST_STRING;
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
    let source = list_cases::LIST_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_not_equal() {
    let source = list_cases::LIST_NOT_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_greater_than() {
    let source = list_cases::LIST_GREATER_THAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_less_than() {
    let source = list_cases::LIST_LESS_THAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_greater_than_or_equal() {
    let source = list_cases::LIST_GREATER_THAN_OR_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_less_than_or_equal() {
    let source = list_cases::LIST_LESS_THAN_OR_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_index_boolean() {
    let source = list_cases::LIST_INDEX_BOOLEAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn list_index_byte() {
    let source = list_cases::LIST_INDEX_BYTE;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte(0x2B)));
}

#[test]
fn list_index_character() {
    let source = list_cases::LIST_INDEX_CHARACTER;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character('c')));
}

#[test]
fn list_index_float() {
    let source = list_cases::LIST_INDEX_FLOAT;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float(2.0)));
}

#[test]
fn list_index_integer() {
    let source = list_cases::LIST_INDEX_INTEGER;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(1)));
}

#[test]
fn list_index_string() {
    let source = list_cases::LIST_INDEX_STRING;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::string("baz".to_string())));
}

#[test]
fn local_list_boolean() {
    let source = list_cases::LOCAL_LIST_BOOLEAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean_list(vec![true, false, true])));
}

#[test]
fn local_list_equal() {
    let source = list_cases::LOCAL_LIST_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_not_equal() {
    let source = list_cases::LOCAL_LIST_NOT_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_greater_than() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_less_than() {
    let source = list_cases::LOCAL_LIST_LESS_THAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_greater_than_or_equal() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN_OR_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}

#[test]
fn local_list_less_than_or_equal() {
    let source = list_cases::LOCAL_LIST_LESS_THAN_OR_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean(true)));
}
