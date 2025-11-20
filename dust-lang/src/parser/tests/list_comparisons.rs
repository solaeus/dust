use crate::{
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::list_cases,
};

#[test]
fn list_equal() {
    let source = list_cases::LIST_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (1, 4),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(7, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (3, 4),
                span: Span(17, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(18, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(24, 29),
            },
        ]
    );
}

#[test]
fn list_not_equal() {
    let source = list_cases::LIST_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (1, 4),
                span: Span(0, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                span: Span(7, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (3, 4),
                span: Span(16, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                span: Span(17, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(23, 27),
            },
        ]
    );
}

#[test]
fn list_greater_than() {
    let source = list_cases::LIST_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (1, 4),
                span: Span(0, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (98, 0),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (97, 0),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (3, 4),
                span: Span(12, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (97, 0),
                span: Span(13, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (98, 0),
                span: Span(18, 21),
            },
        ]
    );
}

#[test]
fn list_less_than() {
    let source = list_cases::LIST_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (1, 4),
                span: Span(0, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: (0, 1065353216),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: (0, 1073741824),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (3, 4),
                span: Span(13, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: (0, 1073741824),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: (0, 1065353216),
                span: Span(19, 22),
            },
        ]
    );
}

#[test]
fn list_greater_than_or_equal() {
    let source = list_cases::LIST_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (1, 4),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(1, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(4, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (3, 4),
                span: Span(10, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(14, 15),
            },
        ]
    );
}

#[test]
fn list_less_than_or_equal() {
    let source = list_cases::LIST_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 1),
                span: Span(0, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (1, 4),
                span: Span(0, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(1, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (1, 1),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (3, 4),
                span: Span(18, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (2, 2),
                span: Span(19, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (3, 3),
                span: Span(26, 31),
            },
        ]
    );
}
