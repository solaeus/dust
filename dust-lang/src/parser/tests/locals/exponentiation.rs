use crate::{
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_byte_exponent() {
    let source = local_cases::LOCAL_BYTE_EXPONENT.to_string();
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
                children: (2, 0),
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
                children: (3, 0),
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
                kind: SyntaxKind::ExponentExpression,
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
fn local_float_exponent() {
    let source = local_cases::LOCAL_FLOAT_EXPONENT.to_string();
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
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(16, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(16, 20),
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
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(28, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(3.0),
                span: Span(36, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(36, 40),
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
                kind: SyntaxKind::ExponentExpression,
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
fn local_integer_exponent() {
    let source = local_cases::LOCAL_INTEGER_EXPONENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 16),
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
                children: (2, 0),
                span: Span(14, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(14, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(17, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(24, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (3, 0),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(30, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentExpression,
                children: (15, 18),
                span: Span(33, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(37, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(37, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(37, 38),
            },
        ]
    );
}

#[test]
fn local_mut_byte_exponent() {
    let source = local_cases::LOCAL_MUT_BYTE_EXPONENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(12, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(19, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(19, 24),
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
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentAssignmentStatement,
                children: (9, 10),
                span: Span(25, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (3, 0),
                span: Span(30, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (13, 0),
                span: Span(36, 37),
            },
        ]
    );
}

#[test]
fn local_mut_float_exponent() {
    let source = local_cases::LOCAL_MUT_FLOAT_EXPONENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(12, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(20, 24),
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
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentAssignmentStatement,
                children: (9, 10),
                span: Span(25, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: (0, 1074266112),
                span: Span(30, 33),
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
                children: (13, 0),
                span: Span(35, 36),
            },
        ]
    );
}

#[test]
fn local_mut_integer_exponent() {
    let source = local_cases::LOCAL_MUT_INTEGER_EXPONENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(12, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(18, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentAssignmentStatement,
                children: (9, 10),
                span: Span(21, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (3, 0),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (13, 0),
                span: Span(29, 30),
            },
        ]
    );
}
