use crate::{
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
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
                children: (5, 1),
                span: Span(0, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (2, 3),
                span: Span(1, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(4, 8),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                span: Span(9, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(15, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (1, 1),
                span: Span(25, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (5, 0),
                span: Span(25, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (5, 1),
                span: Span(0, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (2, 3),
                span: Span(1, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(4, 9),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                span: Span(10, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (1, 1),
                span: Span(25, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (5, 0),
                span: Span(25, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(31, 33),
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
                children: (9, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (6, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (5, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (21, 0),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (9, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (6, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (5, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (21, 0),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (9, 3),
                span: Span(0, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (6, 3),
                span: Span(24, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(33, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(39, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (5, 1),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (21, 0),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (9, 3),
                span: Span(0, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (6, 3),
                span: Span(24, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(33, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(39, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (5, 1),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (21, 0),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (9, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (6, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanOrEqualExpression,
                children: (13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (5, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (21, 0),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (9, 3),
                span: Span(0, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (6, 3),
                span: Span(24, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (13, 16),
                span: Span(27, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(32, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(34, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(40, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (5, 1),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (21, 0),
                span: Span(50, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (15, 3),
                span: Span(0, 84),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (12, 3),
                span: Span(24, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(33, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (9, 3),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (32, 0),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (5, 1),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (21, 0),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (22, 25),
                span: Span(51, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (6, 1),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (24, 0),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (7, 1),
                span: Span(58, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(64, 65),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (8, 1),
                span: Span(73, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (30, 0),
                span: Span(73, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
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
                children: (15, 3),
                span: Span(0, 84),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (12, 3),
                span: Span(24, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (4, 1),
                span: Span(33, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (9, 3),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (32, 0),
                span: Span(48, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (5, 1),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (21, 0),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::EqualExpression,
                children: (22, 25),
                span: Span(51, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (6, 1),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (24, 0),
                span: Span(56, 57),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (7, 1),
                span: Span(58, 68),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(64, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (8, 1),
                span: Span(74, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (30, 0),
                span: Span(74, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
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
                children: (16, 3),
                span: Span(0, 107),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (13, 3),
                span: Span(24, 106),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (11, 1),
                span: Span(33, 91),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (8, 3),
                span: Span(39, 89),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (4, 1),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (19, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (20, 23),
                span: Span(42, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (5, 1),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (22, 0),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (6, 1),
                span: Span(48, 66),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(58, 60),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (7, 1),
                span: Span(72, 89),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (28, 0),
                span: Span(72, 89),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(82, 83),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (12, 1),
                span: Span(97, 106),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (33, 0),
                span: Span(97, 106),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
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
                children: (22, 3),
                span: Span(0, 172),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 4),
                span: Span(1, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(9, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (7, 9),
                span: Span(12, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(16, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (8, 0),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (19, 3),
                span: Span(24, 171),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (12, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (13, 16),
                span: Span(27, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (15, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (17, 1),
                span: Span(33, 156),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (14, 3),
                span: Span(39, 154),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (4, 1),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (19, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (20, 23),
                span: Span(42, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (5, 1),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (22, 0),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (12, 1),
                span: Span(48, 131),
            },
            SyntaxNode {
                kind: SyntaxKind::IfExpression,
                children: (9, 3),
                span: Span(58, 125),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(61, 62),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (6, 1),
                span: Span(61, 62),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (26, 0),
                span: Span(61, 62),
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (27, 28),
                span: Span(61, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(66, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (7, 1),
                span: Span(68, 94),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(82, 84),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (8, 1),
                span: Span(100, 125),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (33, 0),
                span: Span(100, 125),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(114, 115),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (13, 1),
                span: Span(137, 154),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (38, 0),
                span: Span(137, 154),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(147, 148),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (18, 1),
                span: Span(162, 171),
            },
            SyntaxNode {
                kind: SyntaxKind::ElseExpression,
                children: (43, 0),
                span: Span(162, 171),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(168, 169),
            },
        ]
    );
}
