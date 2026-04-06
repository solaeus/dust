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
fn while_expression() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("while x { y }")),
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
            Root.with_child(Span::new(0, 31), SyntaxId(11)),
            FunctionItem
                .with_multiple_children(Span::new(0, 31), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 31), SyntaxId(9)),
            WhileExpression.with_binary_children(Span::new(16, 29), SyntaxId(5), SyntaxId(8)),
            PathExpression.with_child(Span::new(22, 23), SyntaxId(4)),
            PathSegment.empty(Span::new(22, 23)),
            BlockExpression.with_child(Span::new(24, 29), SyntaxId(7)),
            PathExpression.with_child(Span::new(26, 27), SyntaxId(6)),
            PathSegment.empty(Span::new(26, 27)),
        ]
    );
}

#[test]
fn break_empty() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("while true { break; }")),
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
            Root.with_child(Span::new(0, 39), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(0, 39), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 39), SyntaxId(7)),
            WhileExpression.with_binary_children(Span::new(16, 37), SyntaxId(4), SyntaxId(6)),
            BooleanExpression.empty(Span::new(22, 26)),
            BlockExpression.with_child(Span::new(27, 37), SyntaxId(5)),
            BreakExpression.empty(Span::new(29, 35)),
        ]
    );
}

#[test]
fn break_with_value() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("while true { break 42 }")),
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
            Root.with_child(Span::new(0, 41), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 41), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 41), SyntaxId(8)),
            WhileExpression.with_binary_children(Span::new(16, 39), SyntaxId(4), SyntaxId(7)),
            BooleanExpression.empty(Span::new(22, 26)),
            BlockExpression.with_child(Span::new(27, 39), SyntaxId(6)),
            BreakExpression.with_child(Span::new(29, 37), SyntaxId(5)),
            IntegerExpression.empty(Span::new(35, 37)),
        ]
    );
}
