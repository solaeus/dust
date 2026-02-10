use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::list_cases,
};

#[test]
fn list_boolean() {
    let source = list_cases::LIST_BOOLEAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(1)),
                span: Span(0, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(3)),
                span: Span(0, 19),
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
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(14, 18),
            }
        ]
    );
}

#[test]
fn list_byte() {
    let source = list_cases::LIST_BYTE;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(1)),
                span: Span(0, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(3)),
                span: Span(0, 18),
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
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(44),
                span: Span(13, 17),
            }
        ]
    );
}

#[test]
fn list_character() {
    let source = list_cases::LIST_CHARACTER;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(1)),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(3)),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::child(SyntaxId(97)),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::child(SyntaxId(98)),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::child(SyntaxId(99)),
                span: Span(11, 14),
            }
        ]
    );
}

#[test]
fn list_float() {
    let source = list_cases::LIST_FLOAT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(1)),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(3)),
                span: Span(0, 15),
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
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(3.0),
                span: Span(11, 14),
            }
        ]
    );
}

#[test]
fn list_integer() {
    let source = list_cases::LIST_INTEGER;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(1)),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(3)),
                span: Span(0, 9),
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
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(3),
                span: Span(7, 8),
            }
        ]
    );
}

#[test]
fn list_string() {
    let source = list_cases::LIST_STRING;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(1)),
                span: Span(0, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(0), SyntaxId(3)),
                span: Span(0, 21),
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
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"baz"),
                span: Span(15, 20),
            }
        ]
    );
}

#[test]
fn list_index_boolean() {
    let source = list_cases::LIST_INDEX_BOOLEAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(8), SyntaxId(2)),
                span: Span(0, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(3)),
                span: Span(1, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(3)),
                span: Span(9, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(6)),
                span: Span(9, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(23, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(7, 1),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: SyntaxPayload::children(SyntaxId(11), SyntaxId(12)),
                span: Span(30, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(0),
                span: Span(32, 33),
            },
        ]
    );
}

#[test]
fn list_index_byte() {
    let source = list_cases::LIST_INDEX_BYTE;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(8), SyntaxId(2)),
                span: Span(0, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(3)),
                span: Span(1, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(3)),
                span: Span(9, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(6)),
                span: Span(9, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(42),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(43),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(44),
                span: Span(22, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(7, 1),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: SyntaxPayload::children(SyntaxId(11), SyntaxId(12)),
                span: Span(29, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(31, 32),
            },
        ]
    );
}

#[test]
fn list_index_character() {
    let source = list_cases::LIST_INDEX_CHARACTER;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(8), SyntaxId(2)),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(3)),
                span: Span(1, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(3)),
                span: Span(9, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(6)),
                span: Span(9, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('a'),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('b'),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('c'),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(7, 1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: SyntaxPayload::children(SyntaxId(11), SyntaxId(12)),
                span: Span(26, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(28, 29),
            },
        ]
    );
}

#[test]
fn list_index_float() {
    let source = list_cases::LIST_INDEX_FLOAT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(8), SyntaxId(2)),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(3)),
                span: Span(1, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(3)),
                span: Span(9, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(6)),
                span: Span(9, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(1.0),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(3.0),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(7, 1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: SyntaxPayload::children(SyntaxId(11), SyntaxId(12)),
                span: Span(26, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(28, 29),
            },
        ]
    );
}

#[test]
fn list_index_integer() {
    let source = list_cases::LIST_INDEX_INTEGER;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(8), SyntaxId(2)),
                span: Span(0, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(3)),
                span: Span(1, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(3)),
                span: Span(9, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(6)),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(10, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(13, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(3),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(7, 1),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: SyntaxPayload::children(SyntaxId(11), SyntaxId(12)),
                span: Span(20, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(0),
                span: Span(22, 23),
            },
        ]
    );
}

#[test]
fn list_index_string() {
    let source = list_cases::LIST_INDEX_STRING;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(8), SyntaxId(2)),
                span: Span(0, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(3)),
                span: Span(1, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(3)),
                span: Span(9, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(6)),
                span: Span(9, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(10, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"bar"),
                span: Span(17, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"baz"),
                span: Span(24, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(7, 1),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: SyntaxPayload::children(SyntaxId(11), SyntaxId(12)),
                span: Span(32, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(34, 35),
            },
        ]
    );
}

#[test]
fn local_list_boolean() {
    let source = list_cases::LOCAL_LIST_BOOLEAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(8), SyntaxId(2)),
                span: Span(0, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(4), SyntaxId(3)),
                span: Span(1, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(3)),
                span: Span(9, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(6)),
                span: Span(9, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(23, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(7, 1),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(30, 31),
            },
        ]
    );
}

#[test]
fn local_list_equal() {
    let source = list_cases::LOCAL_LIST_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(14), SyntaxId(3)),
                span: Span(0, 54),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(3)),
                span: Span(1, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(2)),
                span: Span(9, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(5)),
                span: Span(9, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(3)),
                span: Span(24, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(28, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(6, 1),
                span: Span(28, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(7), SyntaxId(2)),
                span: Span(32, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(12)),
                span: Span(32, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(33, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(39, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(12, 1),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::children(SyntaxId(17), SyntaxId(20)),
                span: Span(47, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(52, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(13, 1),
                span: Span(52, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(19)),
                span: Span(52, 53),
            },
        ]
    );
}

#[test]
fn local_list_not_equal() {
    let source = list_cases::LOCAL_LIST_NOT_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(14), SyntaxId(3)),
                span: Span(0, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(3)),
                span: Span(1, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(2)),
                span: Span(9, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(5)),
                span: Span(9, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(42),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(43),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(3)),
                span: Span(23, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(6, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(7), SyntaxId(2)),
                span: Span(31, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(12)),
                span: Span(31, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(43),
                span: Span(32, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(42),
                span: Span(38, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(12, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::children(SyntaxId(17), SyntaxId(20)),
                span: Span(45, 51),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(50, 51),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(13, 1),
                span: Span(50, 51),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(19)),
                span: Span(50, 51),
            },
        ]
    );
}

#[test]
fn local_list_greater_than() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(14), SyntaxId(3)),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(3)),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(2)),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(5)),
                span: Span(9, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('b'),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('a'),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(3)),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(6, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(7), SyntaxId(2)),
                span: Span(29, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(12)),
                span: Span(29, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('a'),
                span: Span(30, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('b'),
                span: Span(35, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(12, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::children(SyntaxId(17), SyntaxId(20)),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(13, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(19)),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_list_less_than() {
    let source = list_cases::LOCAL_LIST_LESS_THAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(14), SyntaxId(3)),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(3)),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(2)),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(5)),
                span: Span(9, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(1.0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(3)),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(6, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(7), SyntaxId(2)),
                span: Span(29, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(12)),
                span: Span(29, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(1.0),
                span: Span(30, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(35, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(12, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::children(SyntaxId(17), SyntaxId(20)),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(13, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(19)),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_list_greater_than_or_equal() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(14), SyntaxId(3)),
                span: Span(0, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(3)),
                span: Span(1, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(2)),
                span: Span(9, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(5)),
                span: Span(9, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(10, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(13, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(3)),
                span: Span(17, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(6, 1),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(7), SyntaxId(2)),
                span: Span(25, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(12)),
                span: Span(25, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(12, 1),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::children(SyntaxId(17), SyntaxId(20)),
                span: Span(33, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(38, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(13, 1),
                span: Span(38, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(19)),
                span: Span(38, 39),
            },
        ]
    );
}

#[test]
fn local_list_less_than_or_equal() {
    let source = list_cases::LOCAL_LIST_LESS_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::children(SyntaxId(14), SyntaxId(3)),
                span: Span(0, 56),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(3), SyntaxId(3)),
                span: Span(1, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(1), SyntaxId(2)),
                span: Span(9, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(5)),
                span: Span(9, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(10, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"bar"),
                span: Span(17, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::children(SyntaxId(9), SyntaxId(3)),
                span: Span(25, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(6, 1),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: SyntaxPayload::children(SyntaxId(7), SyntaxId(2)),
                span: Span(33, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(12)),
                span: Span(33, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foo"),
                span: Span(34, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"bar"),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(49, 50),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(12, 1),
                span: Span(49, 50),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(49, 50),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::children(SyntaxId(17), SyntaxId(20)),
                span: Span(49, 55),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(54, 55),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child_indices(13, 1),
                span: Span(54, 55),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(19)),
                span: Span(54, 55),
            },
        ]
    );
}
