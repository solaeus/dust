use crate::{
    instruction::OperandType,
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_byte_subtraction() {
    let source = local_cases::LOCAL_BYTE_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 47),
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
                children: (44, 0),
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
                children: (2, 0),
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
                kind: SyntaxKind::SubtractionExpression,
                children: (9, 10),
                span: Span(41, 46),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(45, 46),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_float_subtraction() {
    let source = local_cases::LOCAL_FLOAT_SUBTRACTION.to_string();
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
                children: SyntaxNode::encode_float(44.0),
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
                span: Span(22, 41),
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
                children: SyntaxNode::encode_float(2.0),
                span: Span(37, 40),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(37, 41),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(42, 43),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (9, 10),
                span: Span(42, 47),
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
fn local_integer_subtraction() {
    let source = local_cases::LOCAL_INTEGER_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 40),
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
                children: (44, 0),
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
                span: Span(18, 33),
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
                children: (2, 0),
                span: Span(31, 32),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(31, 33),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(34, 35),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (9, 10),
                span: Span(34, 39),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(38, 39),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_mut_byte_subtraction() {
    let source = local_cases::LOCAL_MUT_BYTE_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 38),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 24),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(12, 16),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (44, 0),
                span: Span(19, 23),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(19, 24),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(25, 26),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionAssignmentExpression,
                children: (5, 6),
                span: Span(25, 34),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(25, 35),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(30, 34),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(36, 37),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_mut_float_subtraction() {
    let source = local_cases::LOCAL_MUT_FLOAT_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 38),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 25),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(12, 17),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(44.0),
                span: Span(20, 24),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(20, 25),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(26, 27),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionAssignmentExpression,
                children: (5, 6),
                span: Span(26, 34),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(26, 35),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(31, 34),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(36, 37),
                r#type: OperandType::NONE,
            },
        ]
    );
}

#[test]
fn local_mut_integer_subtraction() {
    let source = local_cases::LOCAL_MUT_INTEGER_SUBTRACTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                span: Span(0, 32),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 21),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(12, 15),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (44, 0),
                span: Span(18, 20),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(18, 21),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(22, 23),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionAssignmentExpression,
                children: (5, 6),
                span: Span(22, 28),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(22, 29),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(27, 28),
                r#type: OperandType::NONE,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(30, 31),
                r#type: OperandType::NONE,
            },
        ]
    );
}
