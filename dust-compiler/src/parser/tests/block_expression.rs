use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxKind::*},
    },
};

#[test]
fn empty() {
    let (syntax_tree, errors) = parse(function_wrapper!("{}"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 20), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 20), SyntaxId(2)),
            BlockExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn item() {
    let (syntax_tree, errors) = parse(function_wrapper!("{ fn foo() {} }"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 33), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(5)),
            BlockExpression.with_single_child(Span::new(16, 31), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(18, 29), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(21, 24)),
            BlockExpression.empty(Span::new(27, 29)),
        ]
    );
}

#[test]
fn statement() {
    let (syntax_tree, errors) = parse(function_wrapper!("{ let x = 42; }"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 33), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(5)),
            BlockExpression.with_single_child(Span::new(16, 31), SyntaxId(4)),
            LetStatement.with_binary_children(Span::new(18, 29), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.empty(Span::new(26, 28)),
        ]
    );
}

#[test]
fn expression() {
    let (syntax_tree, errors) = parse(function_wrapper!("{ x + y }"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 27), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(7)),
            BlockExpression.with_single_child(Span::new(16, 25), SyntaxId(6)),
            AdditionExpression.with_binary_children(Span::new(18, 23), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(18, 19), SyntaxId(2)),
            PathSegment.empty(Span::new(18, 19)),
            PathExpression.with_single_child(Span::new(22, 23), SyntaxId(4)),
            PathSegment.empty(Span::new(22, 23)),
        ]
    );
}

#[test]
fn mixed() {
    let (syntax_tree, errors) = parse(function_wrapper!("{ fn foo() {} let x = 42; x + y }"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 51), SyntaxId(15)),
            FunctionItem.with_binary_children(Span::new(0, 51), SyntaxId(1), SyntaxId(14)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 51), SyntaxId(13)),
            BlockExpression.with_children(Span::new(16, 49), SyntaxChildren::new(0, 3)),
            FunctionItem.with_binary_children(Span::new(18, 29), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(21, 24)),
            BlockExpression.empty(Span::new(27, 29)),
            LetStatement.with_binary_children(Span::new(30, 41), SyntaxId(5), SyntaxId(6)),
            SimplePath.empty(Span::new(34, 35)),
            IntegerExpression.empty(Span::new(38, 40)),
            AdditionExpression.with_binary_children(Span::new(42, 47), SyntaxId(9), SyntaxId(11)),
            PathExpression.with_single_child(Span::new(42, 43), SyntaxId(8)),
            PathSegment.empty(Span::new(42, 43)),
            PathExpression.with_single_child(Span::new(46, 47), SyntaxId(10)),
            PathSegment.empty(Span::new(46, 47)),
        ]
    );
}
