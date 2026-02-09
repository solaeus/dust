use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode, SyntaxPayload},
    tests::local_cases,
};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(3, 2),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
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
                payload: SyntaxPayload::child(4),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(21, 22),
            },
        ]
    );
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(3, 2),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
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
                payload: SyntaxPayload::child(4),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(21, 22),
            },
        ]
    );
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(3, 2),
                span: Span(0, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
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
                payload: SyntaxPayload::child(4),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(20, 21),
            },
        ]
    );
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(3, 2),
                span: Span(0, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
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
                payload: SyntaxPayload::child(4),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(22, 23),
            },
        ]
    );
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(3, 2),
                span: Span(0, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
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
                payload: SyntaxPayload::child(4),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(18, 19),
            },
        ]
    );
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(3, 2),
                span: Span(0, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
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
                payload: SyntaxPayload::child(4),
                span: Span(14, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(24, 25),
            },
        ]
    );
}

#[test]
fn local_function() {
    let source = local_cases::LOCAL_FUNCTION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(3, 2),
                span: Span(0, 72),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(1, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(5, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterTypes,
                payload: SyntaxPayload::binary_children(3, 4294967295),
                span: Span(14, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionType,
                payload: SyntaxPayload::binary_children(4, 5),
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
                payload: SyntaxPayload::binary_children(9, 4294967295),
                span: Span(33, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionSignature,
                payload: SyntaxPayload::binary_children(10, 11),
                span: Span(33, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionExpression,
                payload: SyntaxPayload::binary_children(12, 18),
                span: Span(33, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(19),
                span: Span(33, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterName,
                payload: SyntaxPayload::empty(),
                span: Span(34, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterDefinition,
                payload: SyntaxPayload::binary_children(7, 8),
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
                payload: SyntaxPayload::binary_children(17, 4294967295),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::binary_children(13, 4294967295),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(14),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(15, 16),
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
                payload: SyntaxPayload::binary_children(22, 4294967295),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(23),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::CallExpression,
                payload: SyntaxPayload::binary_children(24, 26),
                span: Span(60, 71),
            },
            SyntaxNode {
                kind: SyntaxKind::CallValueArguments,
                payload: SyntaxPayload::binary_children(25, 4294967295),
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
