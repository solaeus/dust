use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::block_cases,
};

#[test]
fn empty_block() {
    let source = block_cases::EMPTY_BLOCK;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 2),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 0),
                position: Span(0, 2),
                payload: TypeId::NONE.0,
            },
        ]
    );
}

#[test]
fn block_expression() {
    let source = block_cases::BLOCK_EXPRESSION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (1, 1),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                position: Span(0, 6),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(2, 4),
                payload: TypeId::INTEGER.0,
            },
        ]
    );
}

#[test]
fn block_statement() {
    let source = block_cases::BLOCK_STATEMENT;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (1, 1),
                position: Span(0, 20),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                position: Span(0, 20),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(2, 18),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(9, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(15, 17),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 18),
                payload: TypeId::INTEGER.0,
            },
        ]
    );
}

#[test]
fn block_statement_and_expression() {
    let source = block_cases::BLOCK_STATEMENT_AND_EXPRESSION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (2, 1),
                position: Span(0, 26),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 2),
                position: Span(0, 26),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(2, 18),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(9, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(15, 17),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 18),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(19, 20),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (5, 6),
                position: Span(19, 24),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                position: Span(23, 24),
                payload: TypeId::INTEGER.0,
            },
        ]
    );
}
