use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn grouped_expression() {
    let (syntax_tree, errors) = parse(function_wrapper!("(x + y)"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 25), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 25), SyntaxId(7)),
            GroupedExpression.with_single_child(Span::new(16, 23), SyntaxId(6)),
            AdditionExpression.with_binary_children(Span::new(17, 22), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(17, 18), SyntaxId(2)),
            PathSegment.empty(Span::new(17, 18)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(4)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}
