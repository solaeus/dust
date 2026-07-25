use crate::{
    parser::parse,
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn use_item() {
    let (syntax_tree, errors) = parse("use foo;");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 8), SyntaxId(3)),
            UseItem.with_single_child(Span::new(0, 8), SyntaxId(2)),
            Path.with_single_child(Span::new(4, 7), SyntaxId(1)),
            PathSegment.empty(Span::new(4, 7)),
        ]
    );
}
