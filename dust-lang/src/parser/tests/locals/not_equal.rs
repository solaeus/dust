use crate::{
    instruction::OperandType,
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean_not_equal() {
    let source = local_cases::LOCAL_BOOLEAN_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 49),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(15, 19),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 41),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(28, 32),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(35, 40),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(35, 41),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(42, 43),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (9, 10),
                span: Span(42, 48),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(47, 48),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_byte_not_equal() {
    let source = local_cases::LOCAL_BYTE_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 48),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (42, 0),
                span: Span(15, 19),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 40),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(28, 32),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (43, 0),
                span: Span(35, 39),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(35, 40),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(41, 42),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (9, 10),
                span: Span(41, 47),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(46, 47),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_character_not_equal() {
    let source = local_cases::LOCAL_CHARACTER_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 46),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 19),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(8, 12),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('z' as u32, 0),
                span: Span(15, 18),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 19),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(20, 38),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(27, 31),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('{' as u32, 0),
                span: Span(34, 37),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(34, 38),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(39, 40),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (9, 10),
                span: Span(39, 45),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(44, 45),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_float_not_equal() {
    let source = local_cases::LOCAL_FLOAT_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 50),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 21),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(8, 13),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(42.0),
                span: Span(16, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(16, 21),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(22, 42),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(29, 34),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(43.0),
                span: Span(37, 41),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(37, 42),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(43, 44),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (9, 10),
                span: Span(43, 49),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(48, 49),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_integer_not_equal() {
    let source = local_cases::LOCAL_INTEGER_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 42),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 17),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(8, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(14, 16),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(14, 17),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(18, 34),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(25, 28),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (43, 0),
                span: Span(31, 33),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(31, 34),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(35, 36),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (9, 10),
                span: Span(35, 41),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(40, 41),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_string_not_equal() {
    let source = local_cases::LOCAL_STRING_NOT_EQUAL.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 48),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(8, 11),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(14, 19),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(14, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(21, 40),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(28, 31),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(34, 39),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(34, 40),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(41, 42),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::NotEqualExpression,
                children: (9, 10),
                span: Span(41, 47),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(46, 47),
                r#type: OperandType::NONE,
            },
        ]
    );
}
