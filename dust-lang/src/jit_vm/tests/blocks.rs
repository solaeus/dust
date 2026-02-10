use crate::{jit_vm::run_main, tests::block_cases, value::Value};

#[test]
fn empty_block() {
    let source = block_cases::EMPTY_BLOCK;
    let result = run_main(source).unwrap();

    assert_eq!(result, None);
}

#[test]
fn block_expression() {
    let source = block_cases::BLOCK_EXPRESSION;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn block_statement() {
    let source = block_cases::BLOCK_STATEMENT;
    let result = run_main(source).unwrap();

    assert_eq!(result, None);
}

#[test]
fn block_statement_and_expression() {
    let source = block_cases::BLOCK_STATEMENT_AND_EXPRESSION;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(43)));
}

#[test]
fn parent_scope_access() {
    let source = block_cases::PARENT_SCOPE_ACCESS;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn nested_parrent_scope_access() {
    let source = block_cases::NESTED_PARRENT_SCOPE_ACCESS;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}

#[test]
fn scope_shadowing() {
    let source = block_cases::SCOPE_SHADOWING;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(43)));
}

#[test]
fn scope_deshadowing() {
    let source = block_cases::SCOPE_DESHADOWING;
    let result = run_main(source).unwrap();

    assert_eq!(result, Some(Value::integer(42)));
}
