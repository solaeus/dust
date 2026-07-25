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
    let (syntax_tree, errors) = parse("const X: i64 = 42;");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 18), SyntaxId(4)),
            ConstItem.with_children(Span::new(0, 18), SyntaxChildren::new(0, 3)),
            SimplePath.empty(Span::new(6, 7)),
            I64Type.empty(Span::new(9, 12)),
            IntegerExpression.empty(Span::new(15, 17)),
        ]
    );
}

#[test]
fn pub_const() {
    let (syntax_tree, errors) = parse("pub const X: i64 = 42;");

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 22), SyntaxId(4)),
            ConstItem
                .with_children(Span::new(4, 22), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::PUBLIC),
            SimplePath.empty(Span::new(10, 11)),
            I64Type.empty(Span::new(13, 16)),
            IntegerExpression.empty(Span::new(19, 21)),
        ]
    );
}
