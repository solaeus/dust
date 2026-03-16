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
fn empty_fields() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("Foo {}")),
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
            Root.with_child(Span::new(0, 24), SyntaxId(9)),
            FunctionItem.with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(7)),
            StructExpression.with_binary_children(Span::new(16, 22), SyntaxId(5), SyntaxId(6)),
            Path.with_child(Span::new(16, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 19)),
            StructExpressionFields.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn fields() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("Foo { x: 42, y: 666 }")),
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
            Root.with_child(Span::new(0, 39), SyntaxId(15)),
            FunctionItem.with_multiple_children(Span::new(0, 39), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 39), SyntaxId(13)),
            StructExpression.with_binary_children(Span::new(16, 37), SyntaxId(5), SyntaxId(12)),
            Path.with_child(Span::new(16, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 19)),
            StructExpressionFields.with_binary_children(Span::new(22, 37), SyntaxId(8), SyntaxId(11)),
            StructExpressionField.with_binary_children(Span::new(22, 28), SyntaxId(6), SyntaxId(7)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.empty(Span::new(25, 27)),
            StructExpressionField.with_binary_children(Span::new(22, 35), SyntaxId(9), SyntaxId(10)),
            SimplePath.empty(Span::new(29, 30)),
            IntegerExpression.empty(Span::new(32, 35)),
        ]
    );
}
