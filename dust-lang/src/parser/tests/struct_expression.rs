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
            Root.with_child(Span::new(0, 24), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(4), SyntaxId(9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(8)),
            StructExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(7)),
            Path.with_child(Span::new(16, 19), SyntaxId(5)),
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
            Root.with_child(Span::new(0, 39), SyntaxId(17)),
            FunctionItem.with_binary_children(Span::new(0, 39), SyntaxId(1), SyntaxId(16)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 39), SyntaxId(4), SyntaxId(15)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 39), SyntaxId(14)),
            StructExpression.with_binary_children(Span::new(16, 37), SyntaxId(6), SyntaxId(13)),
            Path.with_child(Span::new(16, 19), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 19)),
            StructExpressionFields.with_binary_children(
                Span::new(22, 37),
                SyntaxId(9),
                SyntaxId(12)
            ),
            StructDeclartionField.with_binary_children(Span::new(22, 28), SyntaxId(7), SyntaxId(8)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.with_value(Span::new(25, 27), SyntaxPayload::encode_integer(42)),
            StructDeclartionField.with_binary_children(
                Span::new(22, 35),
                SyntaxId(10),
                SyntaxId(11)
            ),
            SimplePath.empty(Span::new(29, 30)),
            IntegerExpression.with_value(Span::new(32, 35), SyntaxPayload::encode_integer(666)),
        ]
    );
}
