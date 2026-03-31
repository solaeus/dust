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
fn r#if() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("if condition { x + y }")),
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
            Root.with_child(Span::new(0, 40), SyntaxId(14)),
            FunctionItem
                .with_multiple_children(Span::new(0, 40), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 40), SyntaxId(12)),
            IfExpression.with_binary_children(Span::new(16, 38), SyntaxId(5), SyntaxId(11)),
            PathExpression.with_child(Span::new(19, 28), SyntaxId(4)),
            PathSegment.empty(Span::new(19, 28)),
            BlockExpression.with_child(Span::new(29, 38), SyntaxId(10)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(7), SyntaxId(9)),
            PathExpression.with_child(Span::new(31, 32), SyntaxId(6)),
            PathSegment.empty(Span::new(31, 32)),
            PathExpression.with_child(Span::new(35, 36), SyntaxId(8)),
            PathSegment.empty(Span::new(35, 36)),
        ]
    );
}

#[test]
fn if_else() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("if condition { x + y } else { x - y }")),
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
            Root.with_child(Span::new(0, 55), SyntaxId(20)),
            FunctionItem
                .with_multiple_children(Span::new(0, 55), SyntaxPayload::child_indices(3, 6)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 55), SyntaxId(18)),
            IfExpression
                .with_multiple_children(Span::new(16, 53), SyntaxPayload::child_indices(0, 3)),
            PathExpression.with_child(Span::new(19, 28), SyntaxId(4)),
            PathSegment.empty(Span::new(19, 28)),
            BlockExpression.with_child(Span::new(29, 38), SyntaxId(10)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(7), SyntaxId(9)),
            PathExpression.with_child(Span::new(31, 32), SyntaxId(6)),
            PathSegment.empty(Span::new(31, 32)),
            PathExpression.with_child(Span::new(35, 36), SyntaxId(8)),
            PathSegment.empty(Span::new(35, 36)),
            BlockExpression.with_child(Span::new(44, 53), SyntaxId(16)),
            SubtractionExpression.with_binary_children(
                Span::new(46, 51),
                SyntaxId(13),
                SyntaxId(15)
            ),
            PathExpression.with_child(Span::new(46, 47), SyntaxId(12)),
            PathSegment.empty(Span::new(46, 47)),
            PathExpression.with_child(Span::new(50, 51), SyntaxId(14)),
            PathSegment.empty(Span::new(50, 51)),
        ]
    );
}

#[test]
fn if_else_if() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!(
            "if left { x + y } else if right { x - y } else { x * y }"
        )),
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
            Root.with_child(Span::new(0, 74), SyntaxId(29)),
            FunctionItem
                .with_multiple_children(Span::new(0, 74), SyntaxPayload::child_indices(6, 9)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 74), SyntaxId(27)),
            IfExpression
                .with_multiple_children(Span::new(16, 72), SyntaxPayload::child_indices(3, 6)),
            PathExpression.with_child(Span::new(19, 23), SyntaxId(4)),
            PathSegment.empty(Span::new(19, 23)),
            BlockExpression.with_child(Span::new(24, 33), SyntaxId(10)),
            AdditionExpression.with_binary_children(Span::new(26, 31), SyntaxId(7), SyntaxId(9)),
            PathExpression.with_child(Span::new(26, 27), SyntaxId(6)),
            PathSegment.empty(Span::new(26, 27)),
            PathExpression.with_child(Span::new(30, 31), SyntaxId(8)),
            PathSegment.empty(Span::new(30, 31)),
            IfExpression
                .with_multiple_children(Span::new(39, 72), SyntaxPayload::child_indices(0, 3)),
            PathExpression.with_child(Span::new(42, 47), SyntaxId(12)),
            PathSegment.empty(Span::new(42, 47)),
            BlockExpression.with_child(Span::new(48, 57), SyntaxId(18)),
            SubtractionExpression.with_binary_children(
                Span::new(50, 55),
                SyntaxId(15),
                SyntaxId(17)
            ),
            PathExpression.with_child(Span::new(50, 51), SyntaxId(14)),
            PathSegment.empty(Span::new(50, 51)),
            PathExpression.with_child(Span::new(54, 55), SyntaxId(16)),
            PathSegment.empty(Span::new(54, 55)),
            BlockExpression.with_child(Span::new(63, 72), SyntaxId(24)),
            MultiplicationExpression.with_binary_children(
                Span::new(65, 70),
                SyntaxId(21),
                SyntaxId(23)
            ),
            PathExpression.with_child(Span::new(65, 66), SyntaxId(20)),
            PathSegment.empty(Span::new(65, 66)),
            PathExpression.with_child(Span::new(69, 70), SyntaxId(22)),
            PathSegment.empty(Span::new(69, 70)),
        ]
    );
}
