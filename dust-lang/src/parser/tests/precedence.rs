use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxChildren},
    },
};

#[test]
fn addition_and_multiplication() {
    let parser = Parser::new(
        FileId::MAIN,
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
            Root.with_single_child(Span::new(0, 27), SyntaxId(14)),
            FunctionItem.with_children(Span::new(0, 27), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(12)),
            AdditionExpression.with_binary_children(Span::new(16, 25), SyntaxId(6), SyntaxId(11)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            MultiplicationExpression.with_binary_children(
                Span::new(20, 25),
                SyntaxId(8),
                SyntaxId(10)
            ),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(7)),
            PathSegment.empty(Span::new(20, 21)),
            PathExpression.with_single_child(Span::new(24, 25), SyntaxId(9)),
            PathSegment.empty(Span::new(24, 25)),
        ]
    );
}

#[test]
fn right_associative_exponentiation() {
    let parser = Parser::new(
        FileId::MAIN,
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
            Root.with_single_child(Span::new(0, 27), SyntaxId(14)),
            FunctionItem.with_children(Span::new(0, 27), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(12)),
            ExponentExpression.with_binary_children(Span::new(16, 25), SyntaxId(6), SyntaxId(11)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            ExponentExpression.with_binary_children(Span::new(20, 25), SyntaxId(8), SyntaxId(10)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(7)),
            PathSegment.empty(Span::new(20, 21)),
            PathExpression.with_single_child(Span::new(24, 25), SyntaxId(9)),
            PathSegment.empty(Span::new(24, 25)),
        ]
    );
}
