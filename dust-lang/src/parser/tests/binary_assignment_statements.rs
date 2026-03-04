use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
};

#[test]
fn add_assign() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("x += 42;")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(9)),
            AdditionAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(7),
                SyntaxId(8)
            ),
            Path.with_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn subtract_assign() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("x -= 42;")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(9)),
            SubtractionAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(7),
                SyntaxId(8)
            ),
            Path.with_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn multiply_assign() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("x *= 42;")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(9)),
            MultiplicationAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(7),
                SyntaxId(8)
            ),
            Path.with_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn divide_assign() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("x /= 42;")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(9)),
            DivisionAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(7),
                SyntaxId(8)
            ),
            Path.with_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn modulo_assign() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("x %= 42;")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(9)),
            ModuloAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(7),
                SyntaxId(8)
            ),
            Path.with_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn power_assign() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("x ^= 42;")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(4), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(9)),
            ExponentAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(7),
                SyntaxId(8)
            ),
            Path.with_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}
