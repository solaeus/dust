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
fn empty() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("{}")),
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
            Root.with_child(Span::new(0, 20), SyntaxId(6)),
            FunctionItem
                .with_multiple_children(Span::new(0, 20), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(4)),
            BlockExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn item() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("{ fn foo() {} }")),
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
            Root.with_child(Span::new(0, 33), SyntaxId(11)),
            FunctionItem
                .with_multiple_children(Span::new(0, 33), SyntaxPayload::child_indices(3, 6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(9)),
            BlockExpression.with_child(Span::new(16, 31), SyntaxId(8)),
            FunctionItem
                .with_multiple_children(Span::new(18, 29), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(21, 24)),
            FunctionParameters.with_child(Span::new(18, 26), SyntaxId(5)),
            ValueParameters.empty(Span::new(18, 26)),
            BlockExpression.empty(Span::new(27, 29)),
        ]
    );
}

#[test]
fn statement() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("{ let x = 42; }")),
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
            Root.with_child(Span::new(0, 33), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 33), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(7)),
            BlockExpression.with_child(Span::new(16, 31), SyntaxId(6)),
            LetStatement.with_binary_children(Span::new(18, 29), SyntaxId(4), SyntaxId(5)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.empty(Span::new(26, 28)),
        ]
    );
}

#[test]
fn expression() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("{ x + y }")),
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
            Root.with_child(Span::new(0, 27), SyntaxId(11)),
            FunctionItem
                .with_multiple_children(Span::new(0, 27), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(9)),
            BlockExpression.with_child(Span::new(16, 25), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(18, 23), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(18, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(18, 19)),
            PathExpression.with_child(Span::new(22, 23), SyntaxId(6)),
            PathSegment.empty(Span::new(22, 23)),
        ]
    );
}

#[test]
fn mixed() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("{ fn foo() {} let x = 42; x + y }")),
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
            Root.with_child(Span::new(0, 51), SyntaxId(19)),
            FunctionItem
                .with_multiple_children(Span::new(0, 51), SyntaxPayload::child_indices(6, 9)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 51), SyntaxId(17)),
            BlockExpression
                .with_multiple_children(Span::new(16, 49), SyntaxPayload::child_indices(3, 6)),
            FunctionItem
                .with_multiple_children(Span::new(18, 29), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(21, 24)),
            FunctionParameters.with_child(Span::new(18, 26), SyntaxId(5)),
            ValueParameters.empty(Span::new(18, 26)),
            BlockExpression.empty(Span::new(27, 29)),
            LetStatement.with_binary_children(Span::new(30, 41), SyntaxId(9), SyntaxId(10)),
            SimplePath.empty(Span::new(34, 35)),
            IntegerExpression.empty(Span::new(38, 40)),
            AdditionExpression.with_binary_children(Span::new(42, 47), SyntaxId(13), SyntaxId(15)),
            PathExpression.with_child(Span::new(42, 43), SyntaxId(12)),
            PathSegment.empty(Span::new(42, 43)),
            PathExpression.with_child(Span::new(46, 47), SyntaxId(14)),
            PathSegment.empty(Span::new(46, 47)),
        ]
    );
}
