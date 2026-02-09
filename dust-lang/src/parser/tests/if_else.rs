use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode, SyntaxPayload},
    tests::if_else_cases,
};

#[test]
fn if_else_true() {
    let source = if_else_cases::IF_ELSE_TRUE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(5, 1),
                span: Span(0, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(2, 3),
                span: Span(1, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(4, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(15, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(1, 1),
                span: Span(25, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(5),
                span: Span(25, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(31, 32),
            },
        ]
    );
}

#[test]
fn if_else_false() {
    let source = if_else_cases::IF_ELSE_FALSE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(5, 1),
                span: Span(0, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(2, 3),
                span: Span(1, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(4, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(10, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(1, 1),
                span: Span(25, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(5),
                span: Span(25, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(31, 33),
            },
        ]
    );
}

#[test]
fn if_else_logical_and() {
    let source = if_else_cases::IF_ELSE_LOGICAL_AND.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(9, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(15, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(19, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(19, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(23, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(23, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(30, 65),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::AndExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(33, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(38, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(38, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(38, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(40, 50),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(46, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(56, 65),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(56, 65),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(62, 63),
            },
        ]
    );
}

#[test]
fn if_else_logical_or() {
    let source = if_else_cases::IF_ELSE_LOGICAL_OR.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(9, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(16, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(24, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(24, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(31, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(34, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(34, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(34, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::OrExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(34, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(41, 51),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(47, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(57, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(57, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(63, 64),
            },
        ]
    );
}

#[test]
fn if_else_equal() {
    let source = if_else_cases::IF_ELSE_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(56, 57),
            },
        ]
    );
}

#[test]
fn if_else_not_equal() {
    let source = if_else_cases::IF_ELSE_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(56, 57),
            },
        ]
    );
}

#[test]
fn if_else_less_than() {
    let source = if_else_cases::IF_ELSE_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(24, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(33, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(39, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(55, 56),
            },
        ]
    );
}

#[test]
fn if_else_greater_than() {
    let source = if_else_cases::IF_ELSE_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(24, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(33, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(39, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(55, 56),
            },
        ]
    );
}

#[test]
fn if_else_less_than_equal() {
    let source = if_else_cases::IF_ELSE_LESS_THAN_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(56, 57),
            },
        ]
    );
}

#[test]
fn if_else_greater_than_equal() {
    let source = if_else_cases::IF_ELSE_GREATER_THAN_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(12, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(21),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(56, 57),
            },
        ]
    );
}

#[test]
fn if_else_if_chain_end() {
    let source = if_else_cases::IF_ELSE_IF_CHAIN_END.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(21, 3),
                span: Span(0, 84),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(18, 3),
                span: Span(24, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(33, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(32),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(21),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(22, 25),
                span: Span(51, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(12, 1),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(24),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(13, 1),
                span: Span(58, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(64, 65),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(14, 1),
                span: Span(73, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(30),
                span: Span(73, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(79, 81),
            },
        ]
    );
}

#[test]
fn if_else_if_chain_middle() {
    let source = if_else_cases::IF_ELSE_IF_CHAIN_MIDDLE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(21, 3),
                span: Span(0, 84),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(18, 3),
                span: Span(24, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(33, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(32),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(21),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                payload: SyntaxPayload::binary_children(22, 25),
                span: Span(51, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(12, 1),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(24),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(13, 1),
                span: Span(58, 68),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(64, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(14, 1),
                span: Span(74, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(30),
                span: Span(74, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(80, 81),
            },
        ]
    );
}

#[test]
fn if_else_nested() {
    let source = if_else_cases::IF_ELSE_NESTED.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(22, 3),
                span: Span(0, 107),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(19, 3),
                span: Span(24, 106),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(17, 1),
                span: Span(33, 91),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(14, 3),
                span: Span(39, 89),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(19),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(20, 23),
                span: Span(42, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(22),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(12, 1),
                span: Span(48, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(58, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(13, 1),
                span: Span(72, 89),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(28),
                span: Span(72, 89),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(82, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(18, 1),
                span: Span(97, 106),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(33),
                span: Span(97, 106),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(103, 104),
            },
        ]
    );
}

#[test]
fn if_else_double_nested() {
    let source = if_else_cases::IF_ELSE_DOUBLE_NESTED.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(28, 3),
                span: Span(0, 172),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(3),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(8),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(25, 3),
                span: Span(24, 171),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(12),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(15),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(23, 1),
                span: Span(33, 156),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(20, 3),
                span: Span(39, 154),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(19),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: SyntaxPayload::binary_children(20, 23),
                span: Span(42, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(11, 1),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(22),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(18, 1),
                span: Span(48, 131),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                payload: SyntaxPayload::binary_children(15, 3),
                span: Span(58, 125),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(61, 62),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(12, 1),
                span: Span(61, 62),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(26),
                span: Span(61, 62),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                payload: SyntaxPayload::binary_children(27, 28),
                span: Span(61, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(66, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(13, 1),
                span: Span(68, 94),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(82, 84),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(14, 1),
                span: Span(100, 125),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(33),
                span: Span(100, 125),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(114, 115),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(19, 1),
                span: Span(137, 154),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(38),
                span: Span(137, 154),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(147, 148),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(24, 1),
                span: Span(162, 171),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                payload: SyntaxPayload::child(43),
                span: Span(162, 171),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(168, 169),
            },
        ]
    );
}
