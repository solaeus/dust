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
        Lexer::with_unvalidated_source(function_wrapper!("Foo;")),
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
            Root.with_child(Span::new(0, 22), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 22), SyntaxPayload::child_indices(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature
                .with_multiple_children(Span::new(0, 9), SyntaxPayload::child_indices(0, 1)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 22), SyntaxId(7)),
            ExpressionStatement.with_child(Span::new(16, 20), SyntaxId(6)),
            PathExpression.with_child(Span::new(16, 19), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn fields() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("Foo { x: 42, y: 666 }")),
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
            Root.with_child(Span::new(0, 39), SyntaxId(14)),
            FunctionItem
                .with_multiple_children(Span::new(0, 39), SyntaxPayload::child_indices(5, 8)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature
                .with_multiple_children(Span::new(0, 9), SyntaxPayload::child_indices(0, 1)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 39), SyntaxId(12)),
            StructExpression.with_binary_children(Span::new(16, 37), SyntaxId(10), SyntaxId(11)),
            Path.with_child(Span::new(16, 19), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 19)),
            StructExpressionStructFields
                .with_multiple_children(Span::new(20, 37), SyntaxPayload::child_indices(1, 5)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.empty(Span::new(25, 27)),
            SimplePath.empty(Span::new(29, 30)),
            IntegerExpression.empty(Span::new(32, 35)),
        ]
    );
}
