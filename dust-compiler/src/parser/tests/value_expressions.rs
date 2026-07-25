use crate::{
    function_wrapper,
    parser::parse,
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn boolean() {
    let (syntax_tree, errors) = parse(function_wrapper!("true"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 22), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 22), SyntaxId(2)),
            BooleanExpression
                .empty(Span::new(16, 20))
                .with_flags(SyntaxFlags::TRUE),
        ]
    );
}

#[test]
fn byte() {
    let (syntax_tree, errors) = parse(function_wrapper!("0x2A"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 22), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 22), SyntaxId(2)),
            HexadecimalExpression.empty(Span::new(16, 20)),
        ]
    );
}

#[test]
fn character() {
    let (syntax_tree, errors) = parse(function_wrapper!("'a'"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 21), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 21), SyntaxId(2)),
            CharacterExpression.empty(Span::new(16, 19)),
        ]
    );
}

#[test]
fn float() {
    let (syntax_tree, errors) = parse(function_wrapper!("42.0"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 22), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(0, 22), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 22), SyntaxId(2)),
            FloatExpression.empty(Span::new(16, 20)),
        ],
    );
}

#[test]
fn integer() {
    let (syntax_tree, errors) = parse(function_wrapper!("42"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 20), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 20), SyntaxId(2)),
            IntegerExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn string() {
    let (syntax_tree, errors) = parse(function_wrapper!("\"Hello, world!\""));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 33), SyntaxId(4)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(2)),
            StringExpression.empty(Span::new(16, 31)),
        ]
    );
}

#[test]
fn list() {
    let (syntax_tree, errors) = parse(function_wrapper!("[1, 2, 3]"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 27), SyntaxId(7)),
            FunctionItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(5)),
            ArrayExpression.with_children(Span::new(16, 25), SyntaxChildren::new(0, 3)),
            IntegerExpression.empty(Span::new(17, 18)),
            IntegerExpression.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(23, 24)),
        ]
    );
}

#[test]
fn array_repeat() {
    let (syntax_tree, errors) = parse(function_wrapper!("[0; 3]"));

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(6)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(4)),
            ArrayRepeatExpression.with_binary_children(Span::new(16, 22), SyntaxId(2), SyntaxId(3)),
            IntegerExpression.empty(Span::new(17, 18)),
            IntegerExpression.empty(Span::new(20, 21)),
        ]
    );
}
