use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn as_expression() {
    let (syntax_tree, errors) = parse(function_wrapper!("x as i32"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(5)),
            AsExpression.with_binary_children(Span::new(16, 24), SyntaxId(3), SyntaxId(4)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            I32Type.empty(Span::new(21, 24)),
        ]
    );
}
