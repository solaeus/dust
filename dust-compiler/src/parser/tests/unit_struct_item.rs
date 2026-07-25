use crate::{
    parser::parse,
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn unit_struct() {
    let (syntax_tree, errors) = parse("struct Foo;");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 11), SyntaxId(2)),
            StructItem.with_single_child(Span::new(0, 11), SyntaxId(1)),
            SimplePath.empty(Span::new(7, 10)),
        ]
    );
}
