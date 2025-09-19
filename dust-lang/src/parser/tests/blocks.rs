use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
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
                children: (0, 1),
                span: Span(0, 2),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 0),
                span: Span(0, 2),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (1, 0),
                span: Span(0, 2),
                payload: TypeId::NONE.0,
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
                children: (1, 1),
                span: Span(0, 6),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                span: Span(0, 6),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(2, 4),
                payload: TypeId::INTEGER.0,
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
                children: (1, 1),
                span: Span(0, 20),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                span: Span(0, 20),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (5, 0),
                span: Span(0, 20),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(2, 18),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(9, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(15, 17),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 18),
                payload: TypeId::INTEGER.0,
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
                children: (2, 1),
                span: Span(0, 26),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 2),
                span: Span(0, 26),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(2, 18),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(9, 12),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(15, 17),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(15, 18),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(19, 20),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (5, 6),
                span: Span(19, 24),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(23, 24),
                payload: TypeId::INTEGER.0,
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
                children: (3, 1),
                span: Span(0, 36),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (1, 2),
                span: Span(1, 35),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(7, 23),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(14, 17),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(20, 22),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(20, 23),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                span: Span(28, 33),
                payload: 1,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(30, 31),
                payload: TypeId::INTEGER.0,
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
                children: (5, 1),
                span: Span(0, 100),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (3, 2),
                span: Span(1, 99),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(7, 23),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(14, 17),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (41, 0),
                span: Span(20, 22),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(20, 23),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (1, 2),
                span: Span(28, 97),
                payload: 1,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(38, 53),
                payload: 257,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(45, 48),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(51, 52),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(51, 53),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                span: Span(62, 91),
                payload: 2,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(76, 77),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (9, 10),
                span: Span(76, 81),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(80, 81),
                payload: TypeId::INTEGER.0,
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
                children: (4, 1),
                span: Span(0, 73),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (2, 2),
                span: Span(1, 72),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(7, 23),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(14, 17),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(20, 22),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(20, 23),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 2),
                span: Span(28, 70),
                payload: 1,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(38, 54),
                payload: 257,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(45, 48),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (43, 0),
                span: Span(51, 53),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(51, 54),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                span: Span(63, 64),
                payload: TypeId::INTEGER.0,
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
                children: (4, 1),
                span: Span(0, 68),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (1, 3),
                span: Span(1, 67),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(7, 23),
                payload: 256,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(14, 17),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(20, 22),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                span: Span(20, 23),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (0, 1),
                span: Span(28, 59),
                payload: 1,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (9, 0),
                span: Span(28, 59),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                span: Span(38, 53),
                payload: 257,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(45, 48),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(51, 52),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                span: Span(51, 53),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                span: Span(64, 65),
                payload: TypeId::INTEGER.0,
            },
        ]
    );
}
