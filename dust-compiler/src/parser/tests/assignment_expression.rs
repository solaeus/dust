use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn assignment_expression() {
    let (syntax_tree, errors) = parse(function_wrapper!("x = 42;"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 25), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 25), SyntaxId(6)),
            ExpressionStatement.with_single_child(Span::new(16, 23), SyntaxId(5)),
            AssignmentExpression.with_binary_children(Span::new(16, 22), SyntaxId(3), SyntaxId(4)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(20, 22)),
        ]
    );
}
