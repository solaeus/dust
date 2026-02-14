use crate::{
    instruction::ByteType,
    jit_vm::run_main,
    tests::{create_function_with_call_case, if_else_cases},
    value::Value,
};

#[test]
fn if_else_true() {
    let source = create_function_with_call_case(if_else_cases::IF_ELSE_TRUE, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_false() {
    let source = create_function_with_call_case(if_else_cases::IF_ELSE_FALSE, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_equal() {
    let source = create_function_with_call_case(if_else_cases::IF_ELSE_EQUAL, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_not_equal() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_NOT_EQUAL, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_less_than() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_LESS_THAN, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_greater_than() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_GREATER_THAN, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_less_than_equal() {
    let source = create_function_with_call_case(
        if_else_cases::IF_ELSE_LESS_THAN_EQUAL,
        ByteType::INTEGER,
    );
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_greater_than_equal() {
    let source = create_function_with_call_case(
        if_else_cases::IF_ELSE_GREATER_THAN_EQUAL,
        ByteType::INTEGER,
    );
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_if_chain_end() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_IF_CHAIN_END, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_if_chain_middle() {
    let source = create_function_with_call_case(
        if_else_cases::IF_ELSE_IF_CHAIN_MIDDLE,
        ByteType::INTEGER,
    );
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_nested() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_NESTED, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_double_nested() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_DOUBLE_NESTED, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_logical_and() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_LOGICAL_AND, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_logical_or() {
    let source =
        create_function_with_call_case(if_else_cases::IF_ELSE_LOGICAL_OR, ByteType::INTEGER);
    let result = run_main(&source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
