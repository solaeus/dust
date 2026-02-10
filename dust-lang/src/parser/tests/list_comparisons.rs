use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::list_cases,
};

#[test]
fn list_equal() {
    let source = list_cases::LIST_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(1)),
                span: Span(0, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(2)),
                span: Span(0, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(6)),
                span: Span(0, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(7, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(2), SyntaxId(2)),
                span: Span(17, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(18, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(24, 29),
            },
        ]
    );
}

#[test]
fn list_not_equal() {
    let source = list_cases::LIST_NOT_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(1)),
                span: Span(0, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(2)),
                span: Span(0, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(6)),
                span: Span(0, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(42),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(43),
                span: Span(7, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(2), SyntaxId(2)),
                span: Span(16, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(43),
                span: Span(17, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(42),
                span: Span(23, 27),
            },
        ]
    );
}

#[test]
fn list_greater_than() {
    let source = list_cases::LIST_GREATER_THAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(1)),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(2)),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(6)),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::child(SyntaxId(98)),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::child(SyntaxId(97)),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(2), SyntaxId(2)),
                span: Span(13, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::child(SyntaxId(97)),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::child(SyntaxId(98)),
                span: Span(19, 22),
            },
        ]
    );
}

#[test]
fn list_less_than() {
    let source = list_cases::LIST_LESS_THAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(1)),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(2)),
                span: Span(0, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(6)),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(1.0),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(2), SyntaxId(2)),
                span: Span(13, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(1.0),
                span: Span(19, 22),
            },
        ]
    );
}

#[test]
fn list_greater_than_or_equal() {
    let source = list_cases::LIST_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(1)),
                span: Span(0, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(2)),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(6)),
                span: Span(0, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(1, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(4, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(2), SyntaxId(2)),
                span: Span(10, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(14, 15),
            },
        ]
    );
}

#[test]
fn list_less_than_or_equal() {
    let source = list_cases::LIST_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(1)),
                span: Span(0, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(2)),
                span: Span(0, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(6)),
                span: Span(0, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(1, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"bar"),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(2), SyntaxId(2)),
                span: Span(18, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(19, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"bar"),
                span: Span(26, 31),
            },
        ]
    );
}
