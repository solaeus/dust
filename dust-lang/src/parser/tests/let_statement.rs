use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[test]
fn let_statement() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let x = 42;")),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 29), SyntaxId(8)),
            FunctionItem
                .with_multiple_children(Span::new(0, 29), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 29), SyntaxId(6)),
            LetStatement.with_binary_children(Span::new(16, 27), SyntaxId(4), SyntaxId(5)),
            SimplePath.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn let_statement_with_type() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let x: i64 = 42;")),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 34), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 34), SyntaxPayload::child_indices(3, 6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 34), SyntaxId(7)),
            LetStatement
                .with_multiple_children(Span::new(16, 32), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(29, 31)),
            I64Type.empty(Span::new(23, 26)),
        ]
    );
}

#[test]
fn let_mut_statement() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let mut x = 42;")),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 33), SyntaxId(8)),
            FunctionItem
                .with_multiple_children(Span::new(0, 33), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(6)),
            {
                let mut node =
                    LetStatement.with_binary_children(Span::new(16, 31), SyntaxId(4), SyntaxId(5));
                node.modifier = true;
                node
            },
            SimplePath.empty(Span::new(24, 25)),
            IntegerExpression.empty(Span::new(28, 30)),
        ]
    );
}

#[test]
fn let_mut_statement_with_type() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let mut x: i64 = 42;")),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 38), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 38), SyntaxPayload::child_indices(3, 6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 38), SyntaxId(7)),
            {
                let mut node = LetStatement
                    .with_multiple_children(Span::new(16, 36), SyntaxPayload::child_indices(0, 3));
                node.modifier = true;
                node
            },
            SimplePath.empty(Span::new(24, 25)),
            IntegerExpression.empty(Span::new(33, 35)),
            I64Type.empty(Span::new(27, 30)),
        ]
    );
}
