use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean_or() {
    let source = local_cases::LOCAL_BOOLEAN_OR;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 3),
                position: Span(0, 49),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(1, 20),
                payload: 256
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                position: Span(8, 12),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                position: Span(15, 19),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(15, 20),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 7),
                position: Span(21, 41),
                payload: 257
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                position: Span(28, 32),
                payload: 0
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                position: Span(35, 40),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (6, 0),
                position: Span(35, 41),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                position: Span(42, 43),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::OrExpression,
                children: (9, 10),
                position: Span(42, 48),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (257, 0),
                position: Span(47, 48),
                payload: TypeId::BOOLEAN.0
            },
        ]
    );
}
