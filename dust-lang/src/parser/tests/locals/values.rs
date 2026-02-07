use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
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
                payload: (5, 2),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                payload: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: (true as u32, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (8, 0),
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
                payload: (5, 2),
                span: Span(0, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                payload: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                payload: (42, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(21, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (8, 0),
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
                payload: (5, 2),
                span: Span(0, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                payload: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                payload: ('q' as u32, 0),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(20, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (8, 0),
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
                payload: (5, 2),
                span: Span(0, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                payload: (0, 0),
                span: Span(8, 13),
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                payload: SyntaxNode::encode_float(42.0),
                span: Span(16, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(16, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (8, 0),
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
                payload: (5, 2),
                span: Span(0, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: (0, 0),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (42, 0),
                span: Span(14, 16),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(18, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (8, 0),
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
                payload: (5, 2),
                span: Span(0, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (1, 3),
                span: Span(1, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                payload: (0, 0),
                span: Span(8, 11),
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                payload: (0, 0),
                span: Span(14, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(14, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(24, 25),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (8, 0),
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
                payload: (10, 2),
                span: Span(0, 72),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: (5, 3),
                span: Span(1, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(5, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(5, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterTypes,
                payload: (1, 1),
                span: Span(14, 21),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionType,
                payload: (4, 5),
                span: Span(14, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: (0, 0),
                span: Span(17, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: (0, 0),
                span: Span(25, 28),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParametersDefinition,
                payload: (2, 1),
                span: Span(33, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionSignature,
                payload: (10, 11),
                span: Span(33, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::FunctionExpression,
                payload: (12, 18),
                span: Span(33, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (19, 0),
                span: Span(33, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterName,
                payload: (0, 0),
                span: Span(34, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::ValueParameterDefinition,
                payload: (7, 8),
                span: Span(34, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: (0, 0),
                span: Span(37, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: (0, 0),
                span: Span(45, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: (4, 1),
                span: Span(49, 58),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (3, 1),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (14, 0),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: (15, 16),
                span: Span(51, 56),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(55, 56),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (8, 1),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (23, 0),
                span: Span(60, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::CallExpression,
                payload: (24, 26),
                span: Span(60, 71),
            },
            SyntaxNode {
                kind: SyntaxKind::CallValueArguments,
                payload: (9, 1),
                span: Span(67, 71),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (41, 0),
                span: Span(68, 70),
            },
        ]
    );
}
