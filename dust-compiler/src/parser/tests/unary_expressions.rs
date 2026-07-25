use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn negation() {
    let (syntax_tree, errors) = parse(function_wrapper!("-x"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 20), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 20), SyntaxId(4)),
            NegationExpression.with_single_child(Span::new(16, 18), SyntaxId(3)),
            PathExpression.with_single_child(Span::new(17, 18), SyntaxId(2)),
            PathSegment.empty(Span::new(17, 18)),
        ]
    );
}

#[test]
fn logical_not() {
    let (syntax_tree, errors) = parse(function_wrapper!("!x"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 20), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 20), SyntaxId(4)),
            NotExpression.with_single_child(Span::new(16, 18), SyntaxId(3)),
            PathExpression.with_single_child(Span::new(17, 18), SyntaxId(2)),
            PathSegment.empty(Span::new(17, 18)),
        ]
    );
}
