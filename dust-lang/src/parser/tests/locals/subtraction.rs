use crate::{
    Span,
    parser::parse_main,
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
                children: (44, 0),
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
                children: (2, 0),
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
                kind: SyntaxKind::SubtractionExpression,
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
                children: SyntaxNode::encode_float(44.0),
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
                span: Span(22, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(29, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(37, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(37, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
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
                children: (44, 0),
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
                span: Span(18, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(25, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(31, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(31, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(34, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionExpression,
                children: (9, 10),
                span: Span(34, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (4, 0),
                span: Span(38, 39),
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
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(12, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (44, 0),
                span: Span(19, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(19, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionAssignmentExpression,
                children: (5, 6),
                span: Span(25, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(25, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(30, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(36, 37),
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
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(12, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(44.0),
                span: Span(20, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(20, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionAssignmentExpression,
                children: (5, 6),
                span: Span(26, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(26, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(31, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(36, 37),
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
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                span: Span(1, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(12, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (44, 0),
                span: Span(18, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(18, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::SubtractionAssignmentExpression,
                children: (5, 6),
                span: Span(22, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (7, 0),
                span: Span(22, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(27, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(30, 31),
            },
        ]
    );
}
