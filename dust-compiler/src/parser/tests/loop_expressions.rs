use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn while_expression() {
    let (syntax_tree, errors) = parse(function_wrapper!("while x { y }"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 31), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 31), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 31), SyntaxId(7)),
            WhileExpression.with_binary_children(Span::new(16, 29), SyntaxId(3), SyntaxId(6)),
            PathExpression.with_single_child(Span::new(22, 23), SyntaxId(2)),
            PathSegment.empty(Span::new(22, 23)),
            BlockExpression.with_single_child(Span::new(24, 29), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(26, 27), SyntaxId(4)),
            PathSegment.empty(Span::new(26, 27)),
        ]
    );
}

#[test]
fn break_empty() {
    let (syntax_tree, errors) = parse(function_wrapper!("while true { break; }"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 39), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 39), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 39), SyntaxId(5)),
            WhileExpression.with_binary_children(Span::new(16, 37), SyntaxId(2), SyntaxId(4)),
            BooleanExpression
                .empty(Span::new(22, 26))
                .with_flags(SyntaxFlags::TRUE),
            BlockExpression.with_single_child(Span::new(27, 37), SyntaxId(3)),
            BreakExpression.empty(Span::new(29, 35)),
        ]
    );
}

#[test]
fn break_with_value() {
    let (syntax_tree, errors) = parse(function_wrapper!("while true { break 42; }"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 42), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 42), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 42), SyntaxId(6)),
            WhileExpression.with_binary_children(Span::new(16, 40), SyntaxId(2), SyntaxId(5)),
            BooleanExpression
                .empty(Span::new(22, 26))
                .with_flags(SyntaxFlags::TRUE),
            BlockExpression.with_single_child(Span::new(27, 40), SyntaxId(4)),
            BreakExpression.with_single_child(Span::new(29, 38), SyntaxId(3)),
            IntegerExpression.empty(Span::new(35, 37)),
        ]
    );
}
