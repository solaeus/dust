use crate::{
    parser::parse,
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn simple() {
    let (syntax_tree, errors) = parse("type Foo = i64;");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 15), SyntaxId(3)),
            TypeItem.with_binary_children(Span::new(0, 15), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(5, 8)),
            I64Type.empty(Span::new(11, 14)),
        ]
    );
}

#[test]
fn with_type_parameters() {
    let (syntax_tree, errors) = parse("type Foo<T> = T;");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 16), SyntaxId(7)),
            TypeItem
                .with_children(Span::new(0, 16), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::TYPE_PARAMETERS),
            SimplePath.empty(Span::new(5, 8)),
            TypeParameters.with_single_child(Span::new(8, 11), SyntaxId(3)),
            TypeParameter.with_single_child(Span::new(9, 10), SyntaxId(2)),
            SimplePath.empty(Span::new(9, 10)),
            TypePath.with_single_child(Span::new(14, 15), SyntaxId(5)),
            PathSegment.empty(Span::new(14, 15)),
        ]
    );
}
