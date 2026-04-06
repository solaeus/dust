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
fn addition_and_multiplication() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("a + b * c")),
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
            Root.with_child(Span::new(0, 27), SyntaxId(13)),
            FunctionItem
                .with_multiple_children(Span::new(0, 27), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(11)),
            AdditionExpression.with_binary_children(Span::new(16, 25), SyntaxId(5), SyntaxId(10)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            MultiplicationExpression.with_binary_children(
                Span::new(20, 25),
                SyntaxId(7),
                SyntaxId(9)
            ),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
            PathExpression.with_child(Span::new(24, 25), SyntaxId(8)),
            PathSegment.empty(Span::new(24, 25)),
        ]
    );
}

#[test]
fn right_associative_exponentiation() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("a ^ b ^ c")),
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
            Root.with_child(Span::new(0, 27), SyntaxId(13)),
            FunctionItem
                .with_multiple_children(Span::new(0, 27), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 27), SyntaxId(11)),
            ExponentExpression.with_binary_children(Span::new(16, 25), SyntaxId(5), SyntaxId(10)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            ExponentExpression.with_binary_children(Span::new(20, 25), SyntaxId(7), SyntaxId(9)),
            PathExpression.with_child(Span::new(20, 21), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
            PathExpression.with_child(Span::new(24, 25), SyntaxId(8)),
            PathSegment.empty(Span::new(24, 25)),
        ]
    );
}
