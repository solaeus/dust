use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean_or() {
    let source = local_cases::LOCAL_BOOLEAN_OR.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (10, 3),
                span: Span(0, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
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
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (1, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (5, 3),
                span: Span(21, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (4, 1),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                children: (0, 0),
                span: Span(28, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                children: (0, 0),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(35, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (8, 1),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::OrExpression,
                children: (15, 18),
                span: Span(42, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (9, 1),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(47, 48),
            },
        ]
    );
}
