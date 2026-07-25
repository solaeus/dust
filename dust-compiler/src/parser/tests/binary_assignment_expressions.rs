use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn add_assign() {
    let (syntax_tree, errors) = parse(function_wrapper!("x += 42;"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(6)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(5)),
            AdditionAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(3),
                SyntaxId(4),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn subtract_assign() {
    let (syntax_tree, errors) = parse(function_wrapper!("x -= 42;"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(6)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(5)),
            SubtractionAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(3),
                SyntaxId(4),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn multiply_assign() {
    let (syntax_tree, errors) = parse(function_wrapper!("x *= 42;"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(6)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(5)),
            MultiplicationAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(3),
                SyntaxId(4),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn divide_assign() {
    let (syntax_tree, errors) = parse(function_wrapper!("x /= 42;"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(6)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(5)),
            DivisionAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(3),
                SyntaxId(4),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn modulo_assign() {
    let (syntax_tree, errors) = parse(function_wrapper!("x %= 42;"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(6)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(5)),
            ModuloAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(3),
                SyntaxId(4),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn power_assign() {
    let (syntax_tree, errors) = parse(function_wrapper!("x ^= 42;"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(6)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(5)),
            ExponentAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(3),
                SyntaxId(4),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}
