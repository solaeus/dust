use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn let_statement() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("let x = 42;")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 29), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 29), SyntaxId(4)),
            LetStatement.with_binary_children(Span::new(16, 27), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn let_statement_with_type() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("let x: i64 = 42;")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 34), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 34), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 34), SyntaxId(5)),
            LetStatement.with_children(Span::new(16, 32), SyntaxChildren::new(0, 3)),
            SimplePath.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(29, 31)),
            I64Type.empty(Span::new(23, 26)),
        ]
    );
}

#[test]
fn let_mut_statement() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("let mut x = 42;")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 33), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(4)),
            LetStatement
                .with_binary_children(Span::new(16, 31), SyntaxId(2), SyntaxId(3))
                .with_flags(SyntaxFlags::MUTABLE),
            SimplePath.empty(Span::new(24, 25)),
            IntegerExpression.empty(Span::new(28, 30)),
        ]
    );
}

#[test]
fn let_mut_statement_with_type() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!(
        "let mut x: i64 = 42;"
    )));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 38), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 38), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 38), SyntaxId(5)),
            LetStatement
                .with_children(Span::new(16, 36), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::MUTABLE),
            SimplePath.empty(Span::new(24, 25)),
            IntegerExpression.empty(Span::new(33, 35)),
            I64Type.empty(Span::new(27, 30)),
        ]
    );
}
