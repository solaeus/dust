use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
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
            Root.with_child(Span::new(0, 40), SyntaxId(16)),
            FunctionItem.with_binary_children(Span::new(0, 40), SyntaxId(1), SyntaxId(15)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 40), SyntaxId(4), SyntaxId(14)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 40), SyntaxId(13)),
            IfExpression.with_binary_children(Span::new(16, 38), SyntaxId(6), SyntaxId(12)),
            PathExpression.with_child(Span::new(19, 28), SyntaxId(5)),
            PathSegment.empty(Span::new(19, 28)),
            BlockExpression.with_child(Span::new(29, 38), SyntaxId(11)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(8), SyntaxId(10)),
            PathExpression.with_child(Span::new(31, 32), SyntaxId(7)),
            PathSegment.empty(Span::new(31, 32)),
            PathExpression.with_child(Span::new(35, 36), SyntaxId(9)),
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
            Root.with_child(Span::new(0, 55), SyntaxId(22)),
            FunctionItem.with_binary_children(Span::new(0, 55), SyntaxId(1), SyntaxId(21)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 55), SyntaxId(4), SyntaxId(20)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 55), SyntaxId(19)),
            IfExpression.with_multiple_children(Span::new(16, 53), 0, 3),
            PathExpression.with_child(Span::new(19, 28), SyntaxId(5)),
            PathSegment.empty(Span::new(19, 28)),
            BlockExpression.with_child(Span::new(29, 38), SyntaxId(11)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(8), SyntaxId(10)),
            PathExpression.with_child(Span::new(31, 32), SyntaxId(7)),
            PathSegment.empty(Span::new(31, 32)),
            PathExpression.with_child(Span::new(35, 36), SyntaxId(9)),
            PathSegment.empty(Span::new(35, 36)),
            BlockExpression.with_child(Span::new(44, 53), SyntaxId(17)),
            SubtractionExpression.with_binary_children(
                Span::new(46, 51),
                SyntaxId(14),
                SyntaxId(16)
            ),
            PathExpression.with_child(Span::new(46, 47), SyntaxId(13)),
            PathSegment.empty(Span::new(46, 47)),
            PathExpression.with_child(Span::new(50, 51), SyntaxId(15)),
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
            Root.with_child(Span::new(0, 74), SyntaxId(31)),
            FunctionItem.with_binary_children(Span::new(0, 74), SyntaxId(1), SyntaxId(30)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 74), SyntaxId(4), SyntaxId(29)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 74), SyntaxId(28)),
            IfExpression.with_multiple_children(Span::new(16, 72), 3, 3),
            PathExpression.with_child(Span::new(19, 23), SyntaxId(5)),
            PathSegment.empty(Span::new(19, 23)),
            BlockExpression.with_child(Span::new(24, 33), SyntaxId(11)),
            AdditionExpression.with_binary_children(Span::new(26, 31), SyntaxId(8), SyntaxId(10)),
            PathExpression.with_child(Span::new(26, 27), SyntaxId(7)),
            PathSegment.empty(Span::new(26, 27)),
            PathExpression.with_child(Span::new(30, 31), SyntaxId(9)),
            PathSegment.empty(Span::new(30, 31)),
            IfExpression.with_multiple_children(Span::new(39, 72), 0, 3),
            PathExpression.with_child(Span::new(42, 47), SyntaxId(13)),
            PathSegment.empty(Span::new(42, 47)),
            BlockExpression.with_child(Span::new(48, 57), SyntaxId(19)),
            SubtractionExpression.with_binary_children(
                Span::new(50, 55),
                SyntaxId(16),
                SyntaxId(18)
            ),
            PathExpression.with_child(Span::new(50, 51), SyntaxId(15)),
            PathSegment.empty(Span::new(50, 51)),
            PathExpression.with_child(Span::new(54, 55), SyntaxId(17)),
            PathSegment.empty(Span::new(54, 55)),
            BlockExpression.with_child(Span::new(63, 72), SyntaxId(25)),
            MultiplicationExpression.with_binary_children(
                Span::new(65, 70),
                SyntaxId(22),
                SyntaxId(24)
            ),
            PathExpression.with_child(Span::new(65, 66), SyntaxId(21)),
            PathSegment.empty(Span::new(65, 66)),
            PathExpression.with_child(Span::new(69, 70), SyntaxId(23)),
            PathSegment.empty(Span::new(69, 70)),
        ]
    );
}
