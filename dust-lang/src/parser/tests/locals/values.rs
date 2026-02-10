use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::local_cases,
};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 23),
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
                kind: SyntaxKind::BooleanType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 20),
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
        ]
    );
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 23),
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
                payload: SyntaxPayload::encode_byte(42),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 20),
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
        ]
    );
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 19),
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
                kind: SyntaxKind::CharacterType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: SyntaxPayload::encode_character('q'),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(20, 21),
            },
        ]
    );
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 21),
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
                payload: SyntaxPayload::encode_float(42.0),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(22, 23),
            },
        ]
    );
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 17),
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
                payload: SyntaxPayload::encode_integer(42),
                span: Span(14, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(18, 19),
            },
        ]
    );
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 23),
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
                kind: SyntaxKind::StringType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: SyntaxPayload::encode_string(b"foobar"),
                span: Span(14, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(14, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(24, 25),
            },
        ]
    );
}

#[test]
fn local_function() {
    let source = local_cases::LOCAL_FUNCTION;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 72),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(5, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterTypes,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(4294967295)),
                span: Span(14, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionType,
                payload: SyntaxPayload::binary_children(SyntaxId(4), SyntaxId(5)),
                span: Span(14, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(17, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(25, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParametersDefinition,
                payload: SyntaxPayload::binary_children(SyntaxId(9), SyntaxId(4294967295)),
                span: Span(33, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionSignature,
                payload: SyntaxPayload::binary_children(SyntaxId(10), SyntaxId(11)),
                span: Span(33, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(12), SyntaxId(18)),
                span: Span(33, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(19)),
                span: Span(33, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterName,
                payload: SyntaxPayload::empty(),
                span: Span(34, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterDefinition,
                payload: SyntaxPayload::binary_children(SyntaxId(7), SyntaxId(8)),
                span: Span(34, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(37, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(45, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(17), SyntaxId(4294967295)),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(SyntaxId(13), SyntaxId(4294967295)),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(15), SyntaxId(16)),
                span: Span(51, 56),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(55, 56),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(SyntaxId(22), SyntaxId(4294967295)),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(23)),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::CallExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(24), SyntaxId(26)),
                span: Span(60, 71),
            },
            SyntaxNode {
                kind: SyntaxKind::CallValueArguments,
                payload: SyntaxPayload::binary_children(SyntaxId(25), SyntaxId(4294967295)),
                span: Span(67, 71),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(41),
                span: Span(68, 70),
            },
        ]
    );
}
