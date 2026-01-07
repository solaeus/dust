use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean_less_than() {
    let source = local_cases::LOCAL_BOOLEAN_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 21),
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
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(15, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(22, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(29, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(36, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(36, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (15, 18),
                span: Span(42, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(46, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(46, 47),
            },
        ]
    );
}

#[test]
fn local_byte_less_than() {
    let source = local_cases::LOCAL_BYTE_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 20),
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
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (41, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(28, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(35, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (15, 18),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_character_less_than() {
    let source = local_cases::LOCAL_CHARACTER_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 19),
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
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('y' as u32, 0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(20, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(27, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('z' as u32, 0),
                span: Span(34, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(34, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (15, 18),
                span: Span(39, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(43, 44),
            },
        ]
    );
}

#[test]
fn local_float_less_than() {
    let source = local_cases::LOCAL_FLOAT_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 21),
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
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(41.0),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(22, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(29, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(37, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(37, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (15, 18),
                span: Span(43, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(47, 48),
            },
        ]
    );
}

#[test]
fn local_integer_less_than() {
    let source = local_cases::LOCAL_INTEGER_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 17),
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
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (41, 0),
                span: Span(14, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(18, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(25, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(31, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(31, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (15, 18),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(39, 40),
            },
        ]
    );
}

#[test]
fn local_string_less_than() {
    let source = local_cases::LOCAL_STRING_LESS_THAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 20),
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
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(14, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(14, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(28, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(34, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(34, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (15, 18),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(45, 46),
            },
        ]
    );
}
