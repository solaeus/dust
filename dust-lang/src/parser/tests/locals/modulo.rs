use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_byte_modulo() {
    let source = local_cases::LOCAL_BYTE_MODULO;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 47),
                payload: TypeId::BYTE.0,
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
                children: (84, 0),
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
                children: (5, 0),
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
                kind: SyntaxKind::ModuloExpression,
                children: (9, 10),
                position: Span(41, 46),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (1, 0),
                position: Span(45, 46),
                payload: TypeId::BYTE.0,
            },
        ]
    );
}

#[test]
fn local_float_modulo() {
    let source = local_cases::LOCAL_FLOAT_MODULO;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 48),
                payload: TypeId::FLOAT.0,
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
                children: SyntaxNode::encode_float(84.0),
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
                position: Span(22, 41),
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
                children: SyntaxNode::encode_float(5.0),
                position: Span(37, 40),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(37, 41),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(42, 43),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloExpression,
                children: (9, 10),
                position: Span(42, 47),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (1, 0),
                position: Span(46, 47),
                payload: TypeId::FLOAT.0,
            },
        ]
    );
}

#[test]
fn local_integer_modulo() {
    let source = local_cases::LOCAL_INTEGER_MODULO;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 40),
                payload: TypeId::INTEGER.0
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
                children: (84, 0),
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
                position: Span(18, 33),
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
                children: (5, 0),
                position: Span(31, 32),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(31, 33),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(34, 35),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloExpression,
                children: (9, 10),
                position: Span(34, 39),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (1, 0),
                position: Span(38, 39),
                payload: TypeId::INTEGER.0
            },
        ]
    );
}

#[test]
fn local_mut_byte_modulo() {
    let source = local_cases::LOCAL_MUT_BYTE_MODULO;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 38),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                position: Span(1, 24),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                position: Span(12, 16),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (84, 0),
                position: Span(19, 23),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(19, 24),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(25, 26),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloAssignmentExpression,
                children: (5, 6),
                position: Span(25, 34),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                position: Span(25, 35),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (5, 0),
                position: Span(30, 34),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(36, 37),
                payload: TypeId::BYTE.0,
            },
        ]
    );
}

#[test]
fn local_mut_float_modulo() {
    let source = local_cases::LOCAL_MUT_FLOAT_MODULO;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 38),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                position: Span(1, 25),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                position: Span(12, 17),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(84.0),
                position: Span(20, 24),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(20, 25),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(26, 27),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloAssignmentExpression,
                children: (5, 6),
                position: Span(26, 34),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                position: Span(26, 35),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(5.0),
                position: Span(31, 34),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(36, 37),
                payload: TypeId::FLOAT.0,
            },
        ]
    );
}

#[test]
fn local_mut_integer_modulo() {
    let source = local_cases::LOCAL_MUT_INTEGER_MODULO;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 32),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                position: Span(1, 21),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(12, 15),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (84, 0),
                position: Span(18, 20),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(18, 21),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(22, 23),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ModuloAssignmentExpression,
                children: (5, 6),
                position: Span(22, 28),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                position: Span(22, 29),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (5, 0),
                position: Span(27, 28),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(30, 31),
                payload: TypeId::INTEGER.0
            },
        ]
    );
}
