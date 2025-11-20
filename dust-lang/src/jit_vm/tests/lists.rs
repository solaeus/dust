use crate::{jit_vm::run_main, tests::list_cases, value::Value};

#[test]
fn list_boolean() {
    let source = list_cases::LIST_BOOLEAN.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::boolean_list(vec![true, false, true])));
}

#[test]
fn list_byte() {
    let source = list_cases::LIST_BYTE.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::byte_list(vec![0x2A, 0x2B, 0x2C])));
}

#[test]
fn list_character() {
    let source = list_cases::LIST_CHARACTER.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::character_list(vec!['a', 'b', 'c'])));
}

#[test]
fn list_float() {
    let source = list_cases::LIST_FLOAT.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::float_list(vec![1.0, 2.0, 3.0])));
}

#[test]
fn list_integer() {
    let source = list_cases::LIST_INTEGER.to_string();
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer_list(vec![1, 2, 3])));
}

#[test]
fn list_string() {
    let source = list_cases::LIST_STRING.to_string();
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
