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
fn addition() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x + y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn subtraction() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x - y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            SubtractionExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn multiplication() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x * y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            MultiplicationExpression.with_binary_children(
                Span::new(16, 21),
                SyntaxId(5),
                SyntaxId(7)
            ),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn division() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x / y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            DivisionExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn modulo() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x % y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            ModuloExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn power() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x ^ y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            ExponentExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn equal() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x == y")),
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
            Root.with_child(Span::new(0, 24), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(8)),
            EqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(6)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn not_equal() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x != y")),
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
            Root.with_child(Span::new(0, 24), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(8)),
            NotEqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(6)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn less_than() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x < y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            LessThanExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn less_than_or_equal() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x <= y")),
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
            Root.with_child(Span::new(0, 24), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(8)),
            LessThanOrEqualExpression.with_binary_children(
                Span::new(16, 22),
                SyntaxId(5),
                SyntaxId(7)
            ),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(6)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn greater_than() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x > y")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(8)),
            GreaterThanExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn greater_than_or_equal() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x >= y")),
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
            Root.with_child(Span::new(0, 24), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(8)),
            GreaterThanOrEqualExpression.with_binary_children(
                Span::new(16, 22),
                SyntaxId(5),
                SyntaxId(7)
            ),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(6)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn logical_and() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x && y")),
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
            Root.with_child(Span::new(0, 24), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(8)),
            AndExpression.with_binary_children(Span::new(16, 22), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(6)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn logical_or() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x || y")),
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
            Root.with_child(Span::new(0, 24), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(8)),
            OrExpression.with_binary_children(Span::new(16, 22), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(6)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}
