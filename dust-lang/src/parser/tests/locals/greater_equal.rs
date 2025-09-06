use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_byte_greater_than_or_equal() {
    let source = local_cases::LOCAL_BYTE_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 48),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 20),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                position: Span(8, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(15, 19),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 20),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                position: Span(21, 40),
                payload: 1,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                position: Span(28, 32),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                position: Span(35, 39),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(35, 40),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(41, 42),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (9, 10),
                position: Span(41, 47),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (1, 0),
                position: Span(46, 47),
                payload: TypeId::BYTE.0,
            },
        ]
    );
}

#[test]
fn local_character_greater_than_or_equal() {
    let source = local_cases::LOCAL_CHARACTER_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 46),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 19),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                position: Span(8, 12),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('z' as u32, 0),
                position: Span(15, 18),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 19),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                position: Span(20, 38),
                payload: 1
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                position: Span(27, 31),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('z' as u32, 0),
                position: Span(34, 37),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(34, 38),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(39, 40),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (9, 10),
                position: Span(39, 45),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (1, 0),
                position: Span(44, 45),
                payload: TypeId::CHARACTER.0
            },
        ]
    );
}

#[test]
fn local_float_greater_than_or_equal() {
    let source = local_cases::LOCAL_FLOAT_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 50),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 21),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                position: Span(8, 13),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(16, 20),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(16, 21),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                position: Span(22, 42),
                payload: 1,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                position: Span(29, 34),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                position: Span(37, 41),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(37, 42),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(43, 44),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (9, 10),
                position: Span(43, 49),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (1, 0),
                position: Span(48, 49),
                payload: TypeId::FLOAT.0,
            },
        ]
    );
}

#[test]
fn local_integer_greater_than_or_equal() {
    let source = local_cases::LOCAL_INTEGER_GREATER_THAN_OR_EQUAL;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 42),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 17),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(8, 11),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(14, 16),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(14, 17),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                position: Span(18, 34),
                payload: 1
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(25, 28),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(31, 33),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(31, 34),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(35, 36),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::GreaterThanOrEqualExpression,
                children: (9, 10),
                position: Span(35, 41),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (1, 0),
                position: Span(40, 41),
                payload: TypeId::INTEGER.0
            },
        ]
    );
}
