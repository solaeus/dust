use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxKind::*},
    },
};

#[test]
fn r#if() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!(
        "if condition { x + y }"
    )));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 40), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 40), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 40), SyntaxId(10)),
            IfExpression.with_binary_children(Span::new(16, 38), SyntaxId(3), SyntaxId(9)),
            PathExpression.with_single_child(Span::new(19, 28), SyntaxId(2)),
            PathSegment.empty(Span::new(19, 28)),
            BlockExpression.with_single_child(Span::new(29, 38), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_single_child(Span::new(31, 32), SyntaxId(4)),
            PathSegment.empty(Span::new(31, 32)),
            PathExpression.with_single_child(Span::new(35, 36), SyntaxId(6)),
            PathSegment.empty(Span::new(35, 36)),
        ]
    );
}

#[test]
fn if_else() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!(
        "if condition { x + y } else { x - y }"
    )));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 55), SyntaxId(18)),
            FunctionItem.with_binary_children(Span::new(0, 55), SyntaxId(1), SyntaxId(17)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 55), SyntaxId(16)),
            IfExpression.with_children(Span::new(16, 53), SyntaxChildren::new(0, 3)),
            PathExpression.with_single_child(Span::new(19, 28), SyntaxId(2)),
            PathSegment.empty(Span::new(19, 28)),
            BlockExpression.with_single_child(Span::new(29, 38), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(31, 36), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_single_child(Span::new(31, 32), SyntaxId(4)),
            PathSegment.empty(Span::new(31, 32)),
            PathExpression.with_single_child(Span::new(35, 36), SyntaxId(6)),
            PathSegment.empty(Span::new(35, 36)),
            BlockExpression.with_single_child(Span::new(44, 53), SyntaxId(14)),
            SubtractionExpression.with_binary_children(
                Span::new(46, 51),
                SyntaxId(11),
                SyntaxId(13),
            ),
            PathExpression.with_single_child(Span::new(46, 47), SyntaxId(10)),
            PathSegment.empty(Span::new(46, 47)),
            PathExpression.with_single_child(Span::new(50, 51), SyntaxId(12)),
            PathSegment.empty(Span::new(50, 51)),
        ]
    );
}

#[test]
fn if_else_if() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!(
        "if left { x + y } else if right { x - y } else { x * y }"
    )));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 74), SyntaxId(27)),
            FunctionItem.with_binary_children(Span::new(0, 74), SyntaxId(1), SyntaxId(26)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 74), SyntaxId(25)),
            IfExpression.with_children(Span::new(16, 72), SyntaxChildren::new(3, 6)),
            PathExpression.with_single_child(Span::new(19, 23), SyntaxId(2)),
            PathSegment.empty(Span::new(19, 23)),
            BlockExpression.with_single_child(Span::new(24, 33), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(26, 31), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_single_child(Span::new(26, 27), SyntaxId(4)),
            PathSegment.empty(Span::new(26, 27)),
            PathExpression.with_single_child(Span::new(30, 31), SyntaxId(6)),
            PathSegment.empty(Span::new(30, 31)),
            IfExpression.with_children(Span::new(39, 72), SyntaxChildren::new(0, 3)),
            PathExpression.with_single_child(Span::new(42, 47), SyntaxId(10)),
            PathSegment.empty(Span::new(42, 47)),
            BlockExpression.with_single_child(Span::new(48, 57), SyntaxId(16)),
            SubtractionExpression.with_binary_children(
                Span::new(50, 55),
                SyntaxId(13),
                SyntaxId(15),
            ),
            PathExpression.with_single_child(Span::new(50, 51), SyntaxId(12)),
            PathSegment.empty(Span::new(50, 51)),
            PathExpression.with_single_child(Span::new(54, 55), SyntaxId(14)),
            PathSegment.empty(Span::new(54, 55)),
            BlockExpression.with_single_child(Span::new(63, 72), SyntaxId(22)),
            MultiplicationExpression.with_binary_children(
                Span::new(65, 70),
                SyntaxId(19),
                SyntaxId(21),
            ),
            PathExpression.with_single_child(Span::new(65, 66), SyntaxId(18)),
            PathSegment.empty(Span::new(65, 66)),
            PathExpression.with_single_child(Span::new(69, 70), SyntaxId(20)),
            PathSegment.empty(Span::new(69, 70)),
        ]
    );
}
