use crate::{jit_vm::run_main, tests::if_else_cases, value::Value};

#[test]
fn if_else_true() {
    let source = if_else_cases::IF_ELSE_TRUE;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_false() {
    let source = if_else_cases::IF_ELSE_FALSE;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_equal() {
    let source = if_else_cases::IF_ELSE_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_not_equal() {
    let source = if_else_cases::IF_ELSE_NOT_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_less_than() {
    let source = if_else_cases::IF_ELSE_LESS_THAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_greater_than() {
    let source = if_else_cases::IF_ELSE_GREATER_THAN;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_less_than_equal() {
    let source = if_else_cases::IF_ELSE_LESS_THAN_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_greater_than_equal() {
    let source = if_else_cases::IF_ELSE_GREATER_THAN_EQUAL;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_if_chain_end() {
    let source = if_else_cases::IF_ELSE_IF_CHAIN_END;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_if_chain_middle() {
    let source = if_else_cases::IF_ELSE_IF_CHAIN_MIDDLE;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_nested() {
    let source = if_else_cases::IF_ELSE_NESTED;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_double_nested() {
    let source = if_else_cases::IF_ELSE_DOUBLE_NESTED;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_logical_and() {
    let source = if_else_cases::IF_ELSE_LOGICAL_AND;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn if_else_logical_or() {
    let source = if_else_cases::IF_ELSE_LOGICAL_OR;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
