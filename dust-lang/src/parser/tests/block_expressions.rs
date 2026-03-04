use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
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
            Root.with_child(Span::new(0, 20), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(3), SyntaxId(5)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
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
            Root.with_child(Span::new(0, 33), SyntaxId(13)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(12)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(3), SyntaxId(11)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(10)),
            BlockExpression.with_child(Span::new(16, 31), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(18, 30), SyntaxId(4), SyntaxId(8)),
            SimplePath.empty(Span::new(21, 24)),
            FunctionExpression.with_binary_children(Span::new(24, 29), SyntaxId(6), SyntaxId(7)),
            FunctionSignature.with_child(Span::new(24, 26), SyntaxId(5)),
            ValueParameters.empty(Span::new(24, 26)),
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
            Root.with_child(Span::new(0, 33), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(3), SyntaxId(8)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(7)),
            BlockExpression.with_child(Span::new(16, 31), SyntaxId(6)),
            LetStatement.with_binary_children(Span::new(18, 29), SyntaxId(4), SyntaxId(5)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.with_value(Span::new(26, 28), SyntaxPayload::encode_integer(42)),
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
            Root.with_child(Span::new(0, 27), SyntaxId(14)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(13)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 27), SyntaxId(3), SyntaxId(12)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(11)),
            BlockExpression.with_child(Span::new(16, 25), SyntaxId(10)),
            AdditionExpression.with_binary_children(Span::new(18, 23), SyntaxId(6), SyntaxId(9)),
            PathExpression.with_child(Span::new(18, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(18, 19)),
            PathExpression.with_child(Span::new(22, 23), SyntaxId(7)),
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
            Root.with_child(Span::new(0, 51), SyntaxId(23)),
            FunctionItem.with_binary_children(Span::new(0, 51), SyntaxId(1), SyntaxId(22)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 51), SyntaxId(3), SyntaxId(21)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 51), SyntaxId(20)),
            BlockExpression.with_multiple_children(Span::new(16, 49), 0, 3),
            FunctionItem.with_binary_children(Span::new(18, 30), SyntaxId(4), SyntaxId(8)),
            SimplePath.empty(Span::new(21, 24)),
            FunctionExpression.with_binary_children(Span::new(24, 29), SyntaxId(6), SyntaxId(7)),
            FunctionSignature.with_child(Span::new(24, 26), SyntaxId(5)),
            ValueParameters.empty(Span::new(24, 26)),
            BlockExpression.empty(Span::new(27, 29)),
            LetStatement.with_binary_children(Span::new(30, 41), SyntaxId(10), SyntaxId(11)),
            SimplePath.empty(Span::new(34, 35)),
            IntegerExpression.with_value(Span::new(38, 40), SyntaxPayload::encode_integer(42)),
            AdditionExpression.with_binary_children(Span::new(42, 47), SyntaxId(15), SyntaxId(18)),
            PathExpression.with_child(Span::new(42, 43), SyntaxId(13)),
            PathSegment.empty(Span::new(42, 43)),
            PathExpression.with_child(Span::new(46, 47), SyntaxId(16)),
            PathSegment.empty(Span::new(46, 47)),
        ]
    );
}
