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
fn boolean() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("true")),
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
            Root.with_child(Span::new(0, 22), SyntaxId(6)),
            FunctionItem
                .with_multiple_children(Span::new(0, 22), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(4)),
            BooleanExpression.empty(Span::new(16, 20)),
        ]
    );
}

#[test]
fn byte() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("0x2A")),
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
            Root.with_child(Span::new(0, 22), SyntaxId(6)),
            FunctionItem
                .with_multiple_children(Span::new(0, 22), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(4)),
            HexadecimalIntegerExpression.empty(Span::new(16, 20)),
        ]
    );
}

#[test]
fn character() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("'a'")),
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
            Root.with_child(Span::new(0, 21), SyntaxId(6)),
            FunctionItem
                .with_multiple_children(Span::new(0, 21), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 21), SyntaxId(4)),
            CharacterExpression.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn float() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("42.0")),
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
            Root.with_child(Span::new(0, 22), SyntaxId(6)),
            FunctionItem
                .with_multiple_children(Span::new(0, 22), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(4)),
            FloatExpression.empty(Span::new(16, 20)),
        ],
    );
}

#[test]
fn integer() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("42")),
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
            IntegerExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn string() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("\"Hello, world!\"")),
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
            Root.with_child(Span::new(0, 33), SyntaxId(6)),
            FunctionItem
                .with_multiple_children(Span::new(0, 33), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(4)),
            StringExpression.empty(Span::new(16, 31)),
        ]
    );
}

#[test]
fn list() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("[1, 2, 3]")),
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
            Root.with_child(Span::new(0, 27), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 27), SyntaxPayload::child_indices(3, 6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(7)),
            ArrayExpression
                .with_multiple_children(Span::new(16, 25), SyntaxPayload::child_indices(0, 3)),
            IntegerExpression.empty(Span::new(17, 18)),
            IntegerExpression.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(23, 24)),
        ]
    );
}
