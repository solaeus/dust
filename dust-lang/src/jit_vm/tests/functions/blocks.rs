use crate::{
    jit_vm::run_main,
    tests::{block_cases, create_function_with_call_case},
    value::Value,
};

#[test]
fn empty_block() {
    let source = create_function_with_call_case(block_cases::EMPTY_BLOCK, "");
    let result = run_main(source).unwrap();

    assert_eq!(result, None);
}

#[test]
fn block_expression() {
    let source = create_function_with_call_case(block_cases::BLOCK_EXPRESSION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn block_statement() {
    let source = create_function_with_call_case(block_cases::BLOCK_STATEMENT, "");
    let result = run_main(source).unwrap();

    assert_eq!(result, None);
}

#[test]
fn block_statement_and_expression() {
    let source = create_function_with_call_case(block_cases::BLOCK_STATEMENT_AND_EXPRESSION, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(43)));
}

#[test]
fn parent_scope_access() {
    let source = create_function_with_call_case(block_cases::PARENT_SCOPE_ACCESS, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn nested_parrent_scope_access() {
    let source = create_function_with_call_case(block_cases::NESTED_PARRENT_SCOPE_ACCESS, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn scope_shadowing() {
    let source = create_function_with_call_case(block_cases::SCOPE_SHADOWING, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(43)));
}

#[test]
fn scope_deshadowing() {
    let source = create_function_with_call_case(block_cases::SCOPE_DESHADOWING, "int");
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
