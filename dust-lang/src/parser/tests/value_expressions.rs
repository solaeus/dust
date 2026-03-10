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
            Root.with_child(Span::new(0, 22), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(4), SyntaxId(6)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(5)),
            BooleanExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_boolean(true)),
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
            Root.with_child(Span::new(0, 22), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(4), SyntaxId(6)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(5)),
            ByteExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_byte(42)),
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
            Root.with_child(Span::new(0, 21), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 21), SyntaxId(4), SyntaxId(6)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 21), SyntaxId(5)),
            CharacterExpression.with_value(Span::new(16, 19), SyntaxPayload::encode_character('a')),
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
            Root.with_child(Span::new(0, 22), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(4), SyntaxId(6)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(5)),
            FloatExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_float(42.0)),
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
            Root.with_child(Span::new(0, 20), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(4), SyntaxId(6)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(5)),
            IntegerExpression.with_value(Span::new(16, 18), SyntaxPayload::encode_integer(42)),
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
            Root.with_child(Span::new(0, 33), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(4), SyntaxId(6)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(5)),
            StringExpression
                .with_value(Span::new(16, 31), SyntaxPayload::encode_string(b"Hello, ")),
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
            Root.with_child(Span::new(0, 27), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 27), SyntaxId(4), SyntaxId(9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(8)),
            ListExpression.with_multiple_children(Span::new(16, 25), 0, 3),
            IntegerExpression.with_value(Span::new(17, 18), SyntaxPayload::encode_integer(1)),
            IntegerExpression.with_value(Span::new(20, 21), SyntaxPayload::encode_integer(2)),
            IntegerExpression.with_value(Span::new(23, 24), SyntaxPayload::encode_integer(3)),
        ]
    );
}

#[test]
fn function() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("fn() {}")),
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
            Root.with_child(Span::new(0, 25), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(4), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(9)),
            FunctionExpression.with_binary_children(Span::new(18, 23), SyntaxId(7), SyntaxId(8)),
            FunctionSignature.with_child(Span::new(18, 20), SyntaxId(6)),
            FunctionParameters.with_child(Span::new(18, 20), SyntaxId(5)),
            ValueParameters.empty(Span::new(18, 20)),
            BlockExpression.empty(Span::new(21, 23)),
        ]
    );
}
