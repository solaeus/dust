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
fn simple() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("foo")),
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
            Root.with_child(Span::new(0, 21), SyntaxId(7)),
            FunctionItem
                .with_multiple_children(Span::new(0, 21), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 21), SyntaxId(5)),
            PathExpression.with_child(Span::new(16, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn multi_segment() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("foo::bar")),
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
            Root.with_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem
                .with_multiple_children(Span::new(0, 26), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(6)),
            PathExpression.with_binary_children(Span::new(16, 24), SyntaxId(4), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 19)),
            PathSegment.empty(Span::new(21, 24)),
        ]
    );
}

#[test]
fn with_type_arguments() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("foo::<Bar>")),
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
            Root.with_child(Span::new(0, 28), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 28), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 28), SyntaxId(8)),
            PathExpression.with_child(Span::new(16, 26), SyntaxId(7)),
            PathSegment.with_child(Span::new(16, 19), SyntaxId(6)),
            TypeArguments.with_child(Span::new(21, 26), SyntaxId(5)),
            TypePath.with_child(Span::new(22, 25), SyntaxId(4)),
            PathSegment.empty(Span::new(22, 25)),
        ]
    );
}

#[test]
fn with_multiple_type_arguments() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("foo::<Bar, Baz>")),
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
            Root.with_child(Span::new(0, 33), SyntaxId(12)),
            FunctionItem
                .with_multiple_children(Span::new(0, 33), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(10)),
            PathExpression.with_child(Span::new(16, 31), SyntaxId(9)),
            PathSegment.with_child(Span::new(16, 19), SyntaxId(8)),
            TypeArguments.with_binary_children(Span::new(21, 31), SyntaxId(5), SyntaxId(7)),
            TypePath.with_child(Span::new(22, 25), SyntaxId(4)),
            PathSegment.empty(Span::new(22, 25)),
            TypePath.with_child(Span::new(27, 30), SyntaxId(6)),
            PathSegment.empty(Span::new(27, 30)),
        ]
    );
}

#[test]
fn multi_segment_with_type_arguments() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("foo::bar::<Baz>")),
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
                .with_multiple_children(Span::new(0, 33), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(9)),
            PathExpression.with_binary_children(Span::new(16, 31), SyntaxId(4), SyntaxId(8)),
            PathSegment.empty(Span::new(16, 19)),
            PathSegment.with_child(Span::new(21, 24), SyntaxId(7)),
            TypeArguments.with_child(Span::new(26, 31), SyntaxId(6)),
            TypePath.with_child(Span::new(27, 30), SyntaxId(5)),
            PathSegment.empty(Span::new(27, 30)),
        ]
    );
}

#[test]
fn type_arguments_on_middle_segment() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("foo::<Bar>::baz")),
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
                .with_multiple_children(Span::new(0, 33), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(9)),
            PathExpression.with_binary_children(Span::new(16, 31), SyntaxId(7), SyntaxId(8)),
            PathSegment.with_child(Span::new(16, 19), SyntaxId(6)),
            TypeArguments.with_child(Span::new(21, 26), SyntaxId(5)),
            TypePath.with_child(Span::new(22, 25), SyntaxId(4)),
            PathSegment.empty(Span::new(22, 25)),
            PathSegment.empty(Span::new(28, 31)),
        ]
    );
}
