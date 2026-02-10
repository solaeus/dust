use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::local_cases,
};

#[test]
fn local_byte_exponent() {
    let source = local_cases::LOCAL_BYTE_EXPONENT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(3)),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(2),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(3)),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: SyntaxPayload::empty(),
                span: Span(28, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(3),
                span: Span(35, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(15), SyntaxId(18)),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_float_exponent() {
    let source = local_cases::LOCAL_FLOAT_EXPONENT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(3)),
                span: Span(0, 47),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(16, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(3)),
                span: Span(21, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: SyntaxPayload::empty(),
                span: Span(28, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(3.0),
                span: Span(36, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(36, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(15), SyntaxId(18)),
                span: Span(41, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(45, 46),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(45, 46),
            },
        ]
    );
}

#[test]
fn local_integer_exponent() {
    let source = local_cases::LOCAL_INTEGER_EXPONENT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(3)),
                span: Span(0, 39),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(14, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(14, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(3)),
                span: Span(17, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(24, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(3),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(30, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(33, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(15), SyntaxId(18)),
                span: Span(33, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(37, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(37, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(37, 38),
            },
        ]
    );
}

#[test]
fn local_mut_byte_exponent() {
    let source = local_cases::LOCAL_MUT_BYTE_EXPONENT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(3)),
                span: Span(0, 38),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(1), SyntaxId(3)),
                span: Span(1, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(2),
                span: Span(19, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(19, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentAssignmentStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(9), SyntaxId(10)),
                span: Span(25, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: SyntaxPayload::encode_byte(3),
                span: Span(30, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(36, 37),
            },
        ]
    );
}

#[test]
fn local_mut_float_exponent() {
    let source = local_cases::LOCAL_MUT_FLOAT_EXPONENT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(3)),
                span: Span(0, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(1), SyntaxId(3)),
                span: Span(1, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::encode_float(2.0),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(20, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentAssignmentStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(9), SyntaxId(10)),
                span: Span(25, 34),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(1074266112)),
                span: Span(30, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(35, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(35, 36),
            },
        ]
    );
}

#[test]
fn local_mut_integer_exponent() {
    let source = local_cases::LOCAL_MUT_INTEGER_EXPONENT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(3)),
                span: Span(0, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(1), SyntaxId(3)),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(12, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(2),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(18, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExponentAssignmentStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(9), SyntaxId(10)),
                span: Span(21, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(3),
                span: Span(26, 27),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(29, 30),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(29, 30),
            },
        ]
    );
}
