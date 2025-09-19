use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_byte_addition() {
    let source = local_cases::LOCAL_BYTE_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 47),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(8, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (40, 0),
                span: Span(15, 19),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 20),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 40),
                payload: 257,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(28, 32),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(35, 39),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(35, 40),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(41, 42),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(41, 46),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(45, 46),
                payload: TypeId::BYTE.0,
            },
        ]
    );
}

#[test]
fn local_float_addition() {
    let source = local_cases::LOCAL_FLOAT_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 48),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 21),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(8, 13),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(40.0),
                span: Span(16, 20),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(16, 21),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(22, 41),
                payload: 257,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(29, 34),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(37, 40),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(37, 41),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(42, 43),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(42, 47),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(46, 47),
                payload: TypeId::FLOAT.0,
            },
        ]
    );
}

#[test]
fn local_integer_addition() {
    let source = local_cases::LOCAL_INTEGER_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 40),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 17),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(8, 11),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (40, 0),
                span: Span(14, 16),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(14, 17),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(18, 33),
                payload: 257
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(25, 28),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(31, 32),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(31, 33),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(34, 35),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(34, 39),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(38, 39),
                payload: TypeId::INTEGER.0
            },
        ]
    );
}

#[test]
fn local_string_concatenation() {
    let source = local_cases::LOCAL_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 47),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(8, 11),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(14, 19),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(14, 20),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 40),
                payload: 257
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(28, 31),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(34, 39),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(34, 40),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(41, 42),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(41, 46),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(45, 46),
                payload: TypeId::STRING.0
            },
        ]
    );
}

#[test]
fn local_character_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 45),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 19),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(8, 12),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(15, 18),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 19),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(20, 38),
                payload: 257
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(27, 31),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(34, 37),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(34, 38),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(39, 40),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(39, 44),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(43, 44),
                payload: TypeId::CHARACTER.0
            },
        ]
    );
}

#[test]
fn local_string_character_concatenation() {
    let source = local_cases::LOCAL_STRING_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 46),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(8, 11),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(14, 19),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(14, 20),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 39),
                payload: 257
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(28, 32),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(35, 38),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(35, 39),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(40, 41),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(40, 45),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(44, 45),
                payload: TypeId::CHARACTER.0
            },
        ]
    );
}

#[test]
fn local_character_string_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 46),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 19),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(8, 12),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(15, 18),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 19),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(20, 39),
                payload: 257
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(27, 30),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(33, 38),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(33, 39),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(40, 41),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(40, 45),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(44, 45),
                payload: TypeId::STRING.0
            },
        ]
    );
}

#[test]
fn local_mut_byte_addition() {
    let source = local_cases::LOCAL_MUT_BYTE_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 38),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 24),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(12, 16),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (40, 0),
                span: Span(19, 23),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(19, 24),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(25, 26),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (5, 6),
                span: Span(25, 34),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(25, 35),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(30, 34),
                payload: TypeId::BYTE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(36, 37),
                payload: TypeId::BYTE.0,
            },
        ]
    );
}

#[test]
fn local_mut_float_addition() {
    let source = local_cases::LOCAL_MUT_FLOAT_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 38),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 25),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(12, 17),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(40.0),
                span: Span(20, 24),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(20, 25),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(26, 27),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (5, 6),
                span: Span(26, 34),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(26, 35),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(31, 34),
                payload: TypeId::FLOAT.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(36, 37),
                payload: TypeId::FLOAT.0,
            },
        ]
    );
}

#[test]
fn local_mut_integer_addition() {
    let source = local_cases::LOCAL_MUT_INTEGER_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 32),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 21),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(12, 15),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (40, 0),
                span: Span(18, 20),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(18, 21),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(22, 23),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (5, 6),
                span: Span(22, 28),
                payload: TypeId::NONE.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(22, 29),
                payload: TypeId::NONE.0
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(27, 28),
                payload: TypeId::INTEGER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(30, 31),
                payload: TypeId::INTEGER.0
            },
        ]
    );
}

#[test]
fn local_mut_string_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 39),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 24),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(12, 15),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(18, 23),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(18, 24),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(25, 26),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (5, 6),
                span: Span(25, 35),
                payload: TypeId::NONE.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(25, 36),
                payload: TypeId::NONE.0
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(30, 35),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(37, 38),
                payload: TypeId::STRING.0
            },
        ]
    );
}

#[test]
fn local_mut_string_character_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 37),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 24),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(12, 15),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(18, 23),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(18, 24),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(25, 26),
                payload: TypeId::STRING.0
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (5, 6),
                span: Span(25, 33),
                payload: TypeId::NONE.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(25, 34),
                payload: TypeId::NONE.0
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(30, 33),
                payload: TypeId::CHARACTER.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(35, 36),
                payload: TypeId::STRING.0
            },
        ]
    );
}
