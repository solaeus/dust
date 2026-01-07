use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
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
                children: (4, 1),
                span: Span(0, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (3, 6),
                span: Span(0, 30),
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
                children: (2, 2),
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
                children: (4, 1),
                span: Span(0, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (3, 6),
                span: Span(0, 28),
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
                children: (2, 2),
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
                children: (4, 1),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (3, 6),
                span: Span(0, 23),
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
                children: (2, 2),
                span: Span(13, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (97, 0),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: (98, 0),
                span: Span(19, 22),
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
                children: (4, 1),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (3, 6),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(1.0),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (2, 2),
                span: Span(13, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(1.0),
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
                children: (4, 1),
                span: Span(0, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (3, 6),
                span: Span(0, 16),
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
                children: (2, 2),
                span: Span(10, 16),
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
                children: (4, 1),
                span: Span(0, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (0, 2),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (3, 6),
                span: Span(0, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(1, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                children: (2, 2),
                span: Span(18, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(19, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(26, 31),
            },
        ]
    );
}
