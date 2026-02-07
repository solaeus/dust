use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
    tests::list_cases,
};

#[test]
fn list_boolean() {
    let source = list_cases::LIST_BOOLEAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (3, 1),
                span: Span(0, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (0, 3),
                span: Span(0, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (0, 0),
                span: Span(7, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(14, 18),
            }
        ]
    );
}

#[test]
fn list_byte() {
    let source = list_cases::LIST_BYTE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (3, 1),
                span: Span(0, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (0, 3),
                span: Span(0, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (42, 0),
                span: Span(1, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (43, 0),
                span: Span(7, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (44, 0),
                span: Span(13, 17),
            }
        ]
    );
}

#[test]
fn list_character() {
    let source = list_cases::LIST_CHARACTER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (3, 1),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (0, 3),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (97, 0),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (98, 0),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (99, 0),
                span: Span(11, 14),
            }
        ]
    );
}

#[test]
fn list_float() {
    let source = list_cases::LIST_FLOAT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (3, 1),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (0, 3),
                span: Span(0, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(1.0),
                span: Span(1, 4),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(2.0),
                span: Span(6, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(3.0),
                span: Span(11, 14),
            }
        ]
    );
}

#[test]
fn list_integer() {
    let source = list_cases::LIST_INTEGER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (3, 1),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (0, 3),
                span: Span(0, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(1, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                span: Span(4, 5),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (3, 0),
                span: Span(7, 8),
            }
        ]
    );
}

#[test]
fn list_string() {
    let source = list_cases::LIST_STRING.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (3, 1),
                span: Span(0, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (0, 3),
                span: Span(0, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(1, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(15, 20),
            }
        ]
    );
}

#[test]
fn list_index_boolean() {
    let source = list_cases::LIST_INDEX_BOOLEAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (8, 2),
                span: Span(0, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (4, 3),
                span: Span(1, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 3),
                span: Span(9, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (6, 0),
                span: Span(9, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (0, 0),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(23, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (10, 0),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: (11, 12),
                span: Span(30, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (0, 0),
                span: Span(32, 33),
            },
        ]
    );
}

#[test]
fn list_index_byte() {
    let source = list_cases::LIST_INDEX_BYTE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (8, 2),
                span: Span(0, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (4, 3),
                span: Span(1, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 3),
                span: Span(9, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (6, 0),
                span: Span(9, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (42, 0),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (43, 0),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (44, 0),
                span: Span(22, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (10, 0),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: (11, 12),
                span: Span(29, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(31, 32),
            },
        ]
    );
}

#[test]
fn list_index_character() {
    let source = list_cases::LIST_INDEX_CHARACTER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (8, 2),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (4, 3),
                span: Span(1, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 3),
                span: Span(9, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (6, 0),
                span: Span(9, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (97, 0),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (98, 0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (99, 0),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (10, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: (11, 12),
                span: Span(26, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                span: Span(28, 29),
            },
        ]
    );
}

#[test]
fn list_index_float() {
    let source = list_cases::LIST_INDEX_FLOAT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (8, 2),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (4, 3),
                span: Span(1, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 3),
                span: Span(9, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (6, 0),
                span: Span(9, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(1.0),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(2.0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(3.0),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (10, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: (11, 12),
                span: Span(26, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(28, 29),
            },
        ]
    );
}

#[test]
fn list_index_integer() {
    let source = list_cases::LIST_INDEX_INTEGER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (8, 2),
                span: Span(0, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (4, 3),
                span: Span(1, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 3),
                span: Span(9, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (6, 0),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(10, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                span: Span(13, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (3, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (10, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: (11, 12),
                span: Span(20, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (0, 0),
                span: Span(22, 23),
            },
        ]
    );
}

#[test]
fn list_index_string() {
    let source = list_cases::LIST_INDEX_STRING.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (8, 2),
                span: Span(0, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (4, 3),
                span: Span(1, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 3),
                span: Span(9, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (6, 0),
                span: Span(9, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(10, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(17, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(24, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (10, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::ListIndexExpression,
                payload: (11, 12),
                span: Span(32, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                span: Span(34, 35),
            },
        ]
    );
}

#[test]
fn local_list_boolean() {
    let source = list_cases::LOCAL_LIST_BOOLEAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (8, 2),
                span: Span(0, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (4, 3),
                span: Span(1, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 3),
                span: Span(9, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (6, 0),
                span: Span(9, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (0, 0),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(23, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (10, 0),
                span: Span(30, 31),
            },
        ]
    );
}

#[test]
fn local_list_equal() {
    let source = list_cases::LOCAL_LIST_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (14, 3),
                span: Span(0, 54),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (3, 3),
                span: Span(1, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 2),
                span: Span(9, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (5, 0),
                span: Span(9, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (0, 0),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (9, 3),
                span: Span(24, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(28, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (6, 1),
                span: Span(28, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (7, 2),
                span: Span(32, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (12, 0),
                span: Span(32, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(33, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (0, 0),
                span: Span(39, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (12, 1),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (16, 0),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: (17, 20),
                span: Span(47, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(52, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (13, 1),
                span: Span(52, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (19, 0),
                span: Span(52, 53),
            },
        ]
    );
}

#[test]
fn local_list_not_equal() {
    let source = list_cases::LOCAL_LIST_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (14, 3),
                span: Span(0, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (3, 3),
                span: Span(1, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 2),
                span: Span(9, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (5, 0),
                span: Span(9, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (42, 0),
                span: Span(10, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (43, 0),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (9, 3),
                span: Span(23, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (6, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (7, 2),
                span: Span(31, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (12, 0),
                span: Span(31, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (43, 0),
                span: Span(32, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (42, 0),
                span: Span(38, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (12, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (16, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: (17, 20),
                span: Span(45, 51),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(50, 51),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (13, 1),
                span: Span(50, 51),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (19, 0),
                span: Span(50, 51),
            },
        ]
    );
}

#[test]
fn local_list_greater_than() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (14, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (3, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 2),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (5, 0),
                span: Span(9, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (98, 0),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (97, 0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (9, 3),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (6, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (7, 2),
                span: Span(29, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (12, 0),
                span: Span(29, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (97, 0),
                span: Span(30, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: (98, 0),
                span: Span(35, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (12, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (16, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: (17, 20),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (13, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (19, 0),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_list_less_than() {
    let source = list_cases::LOCAL_LIST_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (14, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (3, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 2),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (5, 0),
                span: Span(9, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(2.0),
                span: Span(10, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(1.0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (9, 3),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (6, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (7, 2),
                span: Span(29, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (12, 0),
                span: Span(29, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(1.0),
                span: Span(30, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(2.0),
                span: Span(35, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (12, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (16, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: (17, 20),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (13, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (19, 0),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_list_greater_than_or_equal() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (14, 3),
                span: Span(0, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (3, 3),
                span: Span(1, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 2),
                span: Span(9, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (5, 0),
                span: Span(9, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(10, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                span: Span(13, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (9, 3),
                span: Span(17, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (6, 1),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (7, 2),
                span: Span(25, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (12, 0),
                span: Span(25, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (2, 0),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (12, 1),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (16, 0),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: (17, 20),
                span: Span(33, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(38, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (13, 1),
                span: Span(38, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (19, 0),
                span: Span(38, 39),
            },
        ]
    );
}

#[test]
fn local_list_less_than_or_equal() {
    let source = list_cases::LOCAL_LIST_LESS_THAN_OR_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (14, 3),
                span: Span(0, 56),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (3, 3),
                span: Span(1, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (1, 2),
                span: Span(9, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (5, 0),
                span: Span(9, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(10, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(17, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (9, 3),
                span: Span(25, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (6, 1),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::ListExpression,
                payload: (7, 2),
                span: Span(33, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (12, 0),
                span: Span(33, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(34, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(49, 50),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (12, 1),
                span: Span(49, 50),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (16, 0),
                span: Span(49, 50),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: (17, 20),
                span: Span(49, 55),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(54, 55),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (13, 1),
                span: Span(54, 55),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (19, 0),
                span: Span(54, 55),
            },
        ]
    );
}
