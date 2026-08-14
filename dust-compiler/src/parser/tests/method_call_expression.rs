use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{
        SyntaxId,
        node::SyntaxKind::*,
    },
};

#[test]
fn field_access() {
    let (syntax_tree, errors) = parse(function_wrapper!("x.y()"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(5)),
            MethodCallExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(4)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            SimplePath.empty(Span::new(18, 19)),
        ]
    );
}
