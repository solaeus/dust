use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode, SyntaxPayload},
    tests::block_cases,
};

#[test]
fn empty_block() {
    let source = block_cases::EMPTY_BLOCK.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::child(2),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::empty(),
                span: Span(0, 2),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(1),
                span: Span(0, 2),
            },
        ]
    );
}

#[test]
fn block_expression() {
    let source = block_cases::BLOCK_EXPRESSION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(1, 1),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(0, 1),
                span: Span(0, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(2, 4),
            },
        ]
    );
}

#[test]
fn block_statement() {
    let source = block_cases::BLOCK_STATEMENT.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(5, 1),
                span: Span(0, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(4, 1),
                span: Span(0, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(7),
                span: Span(0, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(2, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(6, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(6, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(9, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(15, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(4),
                span: Span(15, 18),
            },
        ]
    );
}

#[test]
fn block_statement_and_expression() {
    let source = block_cases::BLOCK_STATEMENT_AND_EXPRESSION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(7, 1),
                span: Span(0, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(5, 2),
                span: Span(0, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(2, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(6, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(6, 7),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(9, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(15, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(4),
                span: Span(15, 18),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(19, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(19, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(19, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(9, 10),
                span: Span(19, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(23, 24),
            },
        ]
    );
}

#[test]
fn parent_scope_access() {
    let source = block_cases::PARENT_SCOPE_ACCESS.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(8, 1),
                span: Span(0, 36),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(6, 2),
                span: Span(1, 35),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(7, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(4),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(5, 1),
                span: Span(28, 33),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(30, 31),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(8),
                span: Span(30, 31),
            },
        ]
    );
}

#[test]
fn nested_parrent_scope_access() {
    let source = block_cases::NESTED_PARRENT_SCOPE_ACCESS.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(15, 1),
                span: Span(0, 100),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(13, 2),
                span: Span(1, 99),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(7, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(41),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(4),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 2),
                span: Span(28, 97),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(38, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(45, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(10),
                span: Span(51, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 1),
                span: Span(62, 91),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(76, 77),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(76, 77),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(14),
                span: Span(76, 77),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                payload: SyntaxPayload::binary_children(15, 18),
                span: Span(76, 81),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(80, 81),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(80, 81),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(17),
                span: Span(80, 81),
            },
        ]
    );
}

#[test]
fn scope_shadowing() {
    let source = block_cases::SCOPE_SHADOWING.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(13, 1),
                span: Span(0, 73),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(11, 2),
                span: Span(1, 72),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(7, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(4),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(9, 2),
                span: Span(28, 70),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(38, 54),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(45, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(43),
                span: Span(51, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(10),
                span: Span(51, 54),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(63, 64),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(13),
                span: Span(63, 64),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(14),
                span: Span(63, 64),
            },
        ]
    );
}

#[test]
fn scope_deshadowing() {
    let source = block_cases::SCOPE_DESHADOWING.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(13, 1),
                span: Span(0, 68),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(10, 3),
                span: Span(1, 67),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(0, 3),
                span: Span(7, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(1),
                span: Span(11, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(14, 17),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(20, 22),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(4),
                span: Span(20, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(8, 1),
                span: Span(28, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(13),
                span: Span(28, 59),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(3, 3),
                span: Span(38, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(7),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                payload: SyntaxPayload::empty(),
                span: Span(45, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(51, 52),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(10),
                span: Span(51, 53),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(64, 65),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(16),
                span: Span(64, 65),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(16),
                span: Span(64, 65),
            },
        ]
    );
}
