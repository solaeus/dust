use crate::{
    Span,
    parser::parse_main,
    syntax_tree::{SyntaxKind, SyntaxNode},
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
                children: (0, 3),
                span: Span(0, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(28, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(35, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (9, 10),
                span: Span(42, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
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
                children: (0, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 40),
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
                children: (6, 0),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (9, 10),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
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
                children: (0, 3),
                span: Span(0, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('{' as u32, 0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(20, 38),
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
                children: (6, 0),
                span: Span(34, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(39, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (9, 10),
                span: Span(39, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
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
                children: (0, 3),
                span: Span(0, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(43.0),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(22, 42),
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
                children: (6, 0),
                span: Span(37, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(43, 44),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (9, 10),
                span: Span(43, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
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
                children: (0, 3),
                span: Span(0, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (43, 0),
                span: Span(14, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(18, 34),
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
                children: (6, 0),
                span: Span(31, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (9, 10),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
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
                children: (0, 3),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
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
                children: (2, 0),
                span: Span(14, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 40),
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
                children: (6, 0),
                span: Span(34, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanExpression,
                children: (9, 10),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(45, 46),
            },
        ]
    );
}
