use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean_greater_than() {
    let source = local_cases::LOCAL_BOOLEAN_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (10, 3),
                span: Span(0, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
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
                kind: SyntaxKind::BooleanType,
                payload: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (1, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (5, 3),
                span: Span(21, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                payload: (0, 0),
                span: Span(28, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (0, 0),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (10, 0),
                span: Span(35, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (8, 1),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (14, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: (15, 18),
                span: Span(42, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (9, 1),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (17, 0),
                span: Span(46, 47),
            },
        ]
    );
}

#[test]
fn local_byte_greater_than() {
    let source = local_cases::LOCAL_BYTE_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (10, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
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
                kind: SyntaxKind::ByteType,
                payload: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (43, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (5, 3),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: (0, 0),
                span: Span(28, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (42, 0),
                span: Span(35, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (10, 0),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (8, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (14, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: (15, 18),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (9, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (17, 0),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_character_greater_than() {
    let source = local_cases::LOCAL_CHARACTER_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (10, 3),
                span: Span(0, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
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
                kind: SyntaxKind::CharacterType,
                payload: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: ('{' as u32, 0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (5, 3),
                span: Span(20, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                payload: (0, 0),
                span: Span(27, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: ('z' as u32, 0),
                span: Span(34, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (10, 0),
                span: Span(34, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (8, 1),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (14, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: (15, 18),
                span: Span(39, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (9, 1),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (17, 0),
                span: Span(43, 44),
            },
        ]
    );
}

#[test]
fn local_float_greater_than() {
    let source = local_cases::LOCAL_FLOAT_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (10, 3),
                span: Span(0, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 21),
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
                kind: SyntaxKind::FloatType,
                payload: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(43.0),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (5, 3),
                span: Span(22, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: (0, 0),
                span: Span(29, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(42.0),
                span: Span(37, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (10, 0),
                span: Span(37, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (8, 1),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (14, 0),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: (15, 18),
                span: Span(43, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (9, 1),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (17, 0),
                span: Span(47, 48),
            },
        ]
    );
}

#[test]
fn local_integer_greater_than() {
    let source = local_cases::LOCAL_INTEGER_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (10, 3),
                span: Span(0, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 17),
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
                kind: SyntaxKind::IntegerType,
                payload: (0, 0),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (43, 0),
                span: Span(14, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (5, 3),
                span: Span(18, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: (0, 0),
                span: Span(25, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (42, 0),
                span: Span(31, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (10, 0),
                span: Span(31, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (8, 1),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (14, 0),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: (15, 18),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (9, 1),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (17, 0),
                span: Span(39, 40),
            },
        ]
    );
}

#[test]
fn local_string_greater_than() {
    let source = local_cases::LOCAL_STRING_GREATER_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: (10, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
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
                kind: SyntaxKind::StringType,
                payload: (0, 0),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(14, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(14, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (5, 3),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: (0, 0),
                span: Span(28, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(34, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (10, 0),
                span: Span(34, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (8, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (14, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                payload: (15, 18),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (9, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (17, 0),
                span: Span(45, 46),
            },
        ]
    );
}
