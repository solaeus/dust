use crate::{
    parser::parse,
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn empty() {
    let (syntax_tree, errors) = parse("struct Foo {}");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 13), SyntaxId(3)),
            StructItem
                .with_binary_children(Span::new(0, 13), SyntaxId(1), SyntaxId(2))
                .with_flags(SyntaxFlags::FIELDS),
            SimplePath.empty(Span::new(7, 10)),
            NamedFields.empty(Span::new(11, 13)),
        ]
    );
}

#[test]
fn tuple() {
    let (syntax_tree, errors) = parse("struct Foo(i64, i64);");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 21), SyntaxId(5)),
            StructItem
                .with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(4))
                .with_flags(SyntaxFlags::FIELDS),
            SimplePath.empty(Span::new(7, 10)),
            TupleFields.with_binary_children(Span::new(10, 20), SyntaxId(2), SyntaxId(3)),
            I64Type.empty(Span::new(11, 14)),
            I64Type.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn fields() {
    let (syntax_tree, errors) = parse("struct Foo { x: i64, y: i64 }");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 29), SyntaxId(7)),
            StructItem
                .with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(6))
                .with_flags(SyntaxFlags::FIELDS),
            SimplePath.empty(Span::new(7, 10)),
            NamedFields.with_children(Span::new(11, 29), SyntaxChildren::new(0, 4)),
            SimplePath.empty(Span::new(13, 14)),
            I64Type.empty(Span::new(16, 19)),
            SimplePath.empty(Span::new(21, 22)),
            I64Type.empty(Span::new(24, 27)),
        ]
    );
}

#[test]
fn type_parameters() {
    let (syntax_tree, errors) = parse("struct Foo<A, B, C> {}");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 22), SyntaxId(10)),
            StructItem
                .with_children(Span::new(0, 22), SyntaxChildren::new(3, 6))
                .with_flags(SyntaxFlags::TYPE_PARAMETERS.and(SyntaxFlags::FIELDS)),
            SimplePath.empty(Span::new(7, 10)),
            TypeParameters.with_children(Span::new(10, 19), SyntaxChildren::new(0, 3)),
            TypeParameter.with_single_child(Span::new(11, 12), SyntaxId(2)),
            SimplePath.empty(Span::new(11, 12)),
            TypeParameter.with_single_child(Span::new(14, 15), SyntaxId(4)),
            SimplePath.empty(Span::new(14, 15)),
            TypeParameter.with_single_child(Span::new(17, 18), SyntaxId(6)),
            SimplePath.empty(Span::new(17, 18)),
            NamedFields.empty(Span::new(20, 22)),
        ]
    );
}
