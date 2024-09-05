use dust_lang::*;

#[test]
fn block_scope_captures_parent() {
    let source = "let x = 42; { x }";

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}

#[test]
fn block_scope_does_not_capture_child() {
    env_logger::builder().is_test(true).try_init().unwrap();

    let source = "{ let x = 42; } x";

    assert_eq!(
        run(source),
        Err(DustError::analysis(
            [AnalysisError::UndefinedVariable {
                identifier: Node::new(Identifier::new("x"), (16, 17))
            }],
            source
        ))
    );
}

#[test]
fn block_scope_does_not_capture_sibling() {
    let source = "{ let x = 42; } { x }";

    assert_eq!(
        run(source),
        Err(DustError::analysis(
            [AnalysisError::UndefinedVariable {
                identifier: Node::new(Identifier::new("x"), (18, 19))
            }],
            source
        ))
    );
}

#[test]
fn block_scope_does_not_pollute_parent() {
    let source = "let x = 42; { let x = \"foo\"; let x = \"bar\"; } x";

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}
