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
    let (syntax_tree, errors) = parse("fn foo() {}");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 11), SyntaxId(3)),
            FunctionItem.with_binary_children(Span::new(0, 11), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(3, 6)),
            BlockExpression.empty(Span::new(9, 11)),
        ]
    );
}

#[test]
fn value_parameters() {
    let (syntax_tree, errors) = parse("fn foo(x: i64, y: bool) {}");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            FunctionItem
                .with_children(Span::new(0, 26), SyntaxChildren::new(4, 7))
                .with_flags(SyntaxFlags::VALUE_PARAMETERS),
            SimplePath.empty(Span::new(3, 6)),
            ValueParameters.with_children(Span::new(6, 23), SyntaxChildren::new(0, 4)),
            SimplePath.empty(Span::new(7, 8)),
            I64Type.empty(Span::new(10, 13)),
            SimplePath.empty(Span::new(15, 16)),
            BooleanType.empty(Span::new(18, 22)),
            BlockExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn type_parameters() {
    let (syntax_tree, errors) = parse("fn foo<A, B, C>() {}");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 20), SyntaxId(10)),
            FunctionItem
                .with_children(Span::new(0, 20), SyntaxChildren::new(3, 6))
                .with_flags(SyntaxFlags::TYPE_PARAMETERS),
            SimplePath.empty(Span::new(3, 6)),
            TypeParameters.with_children(Span::new(6, 15), SyntaxChildren::new(0, 3)),
            TypeParameter.with_single_child(Span::new(7, 8), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 8)),
            TypeParameter.with_single_child(Span::new(10, 11), SyntaxId(4)),
            SimplePath.empty(Span::new(10, 11)),
            TypeParameter.with_single_child(Span::new(13, 14), SyntaxId(6)),
            SimplePath.empty(Span::new(13, 14)),
            BlockExpression.empty(Span::new(18, 20)),
        ]
    );
}

#[test]
fn return_type() {
    let (syntax_tree, errors) = parse("fn foo() -> i64 {}");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 18), SyntaxId(4)),
            FunctionItem
                .with_children(Span::new(0, 18), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::RETURN_TYPE),
            SimplePath.empty(Span::new(3, 6)),
            I64Type.empty(Span::new(12, 15)),
            BlockExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn mixed() {
    let (syntax_tree, errors) = parse("fn foo<A, B, C>(x: A, y: B) -> C {}");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 35), SyntaxId(19)),
            FunctionItem
                .with_children(Span::new(0, 35), SyntaxChildren::new(7, 12))
                .with_flags(
                    SyntaxFlags::TYPE_PARAMETERS
                        .and(SyntaxFlags::VALUE_PARAMETERS)
                        .and(SyntaxFlags::RETURN_TYPE),
                ),
            SimplePath.empty(Span::new(3, 6)),
            TypeParameters.with_children(Span::new(6, 15), SyntaxChildren::new(0, 3)),
            TypeParameter.with_single_child(Span::new(7, 8), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 8)),
            TypeParameter.with_single_child(Span::new(10, 11), SyntaxId(4)),
            SimplePath.empty(Span::new(10, 11)),
            TypeParameter.with_single_child(Span::new(13, 14), SyntaxId(6)),
            SimplePath.empty(Span::new(13, 14)),
            ValueParameters.with_children(Span::new(15, 27), SyntaxChildren::new(3, 7)),
            SimplePath.empty(Span::new(16, 17)),
            TypePath.with_single_child(Span::new(19, 20), SyntaxId(10)),
            PathSegment.empty(Span::new(19, 20)),
            SimplePath.empty(Span::new(22, 23)),
            TypePath.with_single_child(Span::new(25, 26), SyntaxId(13)),
            PathSegment.empty(Span::new(25, 26)),
            TypePath.with_single_child(Span::new(31, 32), SyntaxId(16)),
            PathSegment.empty(Span::new(31, 32)),
            BlockExpression.empty(Span::new(33, 35)),
        ]
    );
}
