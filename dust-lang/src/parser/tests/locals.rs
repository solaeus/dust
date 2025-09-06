use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 23),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 20),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                position: Span(8, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (true as u32, 0),
                position: Span(15, 19),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 20),
                payload: TypeId::BOOLEAN.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(21, 22),
                payload: TypeId::BOOLEAN.0,
            },
        ]
    );
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 23),
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
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(21, 22),
                payload: TypeId::BYTE.0,
            },
        ]
    );
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 22),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 19),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                position: Span(8, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                position: Span(15, 18),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 19),
                payload: TypeId::CHARACTER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(20, 21),
                payload: TypeId::CHARACTER.0,
            },
        ]
    );
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 24),
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
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(22, 23),
                payload: TypeId::FLOAT.0,
            },
        ]
    );
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 20),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 17),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(8, 11),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(14, 16),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(14, 17),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(18, 19),
                payload: TypeId::INTEGER.0,
            },
        ]
    );
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 29),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 26),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                position: Span(8, 14),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 6),
                position: Span(17, 25),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(17, 26),
                payload: TypeId::STRING.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(27, 28),
                payload: TypeId::STRING.0,
            },
        ]
    );
}

#[test]
fn local_byte_addition() {
    let source = local_cases::LOCAL_BYTE_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            // Entire main with two lets and a final expression
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 47),
                payload: TypeId::BYTE.0,
            },
            // let a: byte = 0x28;
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
                children: (40, 0),
                position: Span(15, 19),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 20),
                payload: TypeId::BYTE.0,
            },
            // let b: byte = 0x02;
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                position: Span(21, 40),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                position: Span(28, 32),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                position: Span(35, 39),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(35, 40),
                payload: TypeId::BYTE.0,
            },
            // a + b
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(41, 42),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                position: Span(41, 46),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(45, 46),
                payload: TypeId::BYTE.0,
            },
        ]
    );
}

#[test]
fn local_float_addition() {
    let source = local_cases::LOCAL_FLOAT_ADDITION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            // Entire main with two lets and a final expression
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 48),
                payload: TypeId::FLOAT.0,
            },
            // let a: float = 40.0;
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
                children: SyntaxNode::encode_float(40.0),
                position: Span(16, 20),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(16, 21),
                payload: TypeId::FLOAT.0,
            },
            // let b: float = 2.0;
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                position: Span(22, 41),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                position: Span(29, 34),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                position: Span(37, 40),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(37, 41),
                payload: TypeId::FLOAT.0,
            },
            // a + b
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(42, 43),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                position: Span(42, 47),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(46, 47),
                payload: TypeId::FLOAT.0,
            },
        ]
    );
}
