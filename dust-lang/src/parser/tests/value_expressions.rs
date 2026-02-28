use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::value_expressions::{BOOLEAN, BYTE, CHARACTER, FLOAT, FUNCTION, INTEGER, LIST, STRING},
};

#[test]
fn boolean() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(BOOLEAN));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 22), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(3), SyntaxId(5)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(4)),
            BooleanExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_boolean(true)),
        ]
    );
}

#[test]
fn byte() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(BYTE));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 22), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(3), SyntaxId(5)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(4)),
            ByteExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_byte(42)),
        ]
    );
}

#[test]
fn character() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(CHARACTER));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 21), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 21), SyntaxId(3), SyntaxId(5)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 21), SyntaxId(4)),
            CharacterExpression.with_value(Span::new(16, 19), SyntaxPayload::encode_character('a')),
        ]
    );
}

#[test]
fn float() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(FLOAT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 22), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 22), SyntaxId(3), SyntaxId(5)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(4)),
            FloatExpression.with_value(Span::new(16, 20), SyntaxPayload::encode_float(42.0)),
        ],
    );
}

#[test]
fn integer() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(INTEGER));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 20), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(3), SyntaxId(5)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(4)),
            IntegerExpression.with_value(Span::new(16, 18), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn string() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(STRING));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 33), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(3), SyntaxId(5)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(4)),
            StringExpression
                .with_value(Span::new(16, 31), SyntaxPayload::encode_string(b"Hello, ")),
        ]
    );
}

#[test]
fn list() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(LIST));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 27), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 27), SyntaxId(3), SyntaxId(8)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(7)),
            ListExpression.with_multiple_children(Span::new(16, 25), 0, 3),
        ]
    );
}

#[test]
fn function() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(FUNCTION));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 25), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(3), SyntaxId(8)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(7)),
            FunctionExpression.with_binary_children(Span::new(18, 23), SyntaxId(5), SyntaxId(6)),
            FunctionSignature.with_child(Span::new(18, 20), SyntaxId(4)),
            ValueParameters.empty(Span::new(18, 20)),
            BlockExpression.empty(Span::new(21, 23)),
        ]
    );
}
