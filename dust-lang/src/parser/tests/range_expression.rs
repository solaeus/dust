use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[test]
fn exclusive() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("1..10")),
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
            Root.with_child(Span::new(0, 23), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature
                .with_multiple_children(Span::new(0, 9), SyntaxPayload::child_indices(0, 1)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 23), SyntaxId(7)),
            RangeExpression.with_binary_children(Span::new(16, 21), SyntaxId(5), SyntaxId(6)),
            IntegerExpression.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(19, 21)),
        ]
    );
}

#[test]
fn inclusive() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("1..=10")),
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
            FunctionItem
                .with_multiple_children(Span::new(0, 24), SyntaxPayload::child_indices(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature
                .with_multiple_children(Span::new(0, 9), SyntaxPayload::child_indices(0, 1)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(7)),
            RangeInclusiveExpression.with_binary_children(
                Span::new(16, 22),
                SyntaxId(5),
                SyntaxId(6)
            ),
            IntegerExpression.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(20, 22)),
        ]
    );
}
