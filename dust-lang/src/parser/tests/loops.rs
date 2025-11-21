use crate::{
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::loop_cases,
};

#[test]
fn while_loop() {
    let source = loop_cases::WHILE_LOOP.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (5, 3),
                span: Span(0, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 4),
                span: Span(1, 15),
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
                kind: SyntaxKind::IntegerExpression,
                children: (0, 0),
                span: Span(13, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (3, 0),
                span: Span(13, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::WhileExpression,
                children: (10, 18),
                span: Span(17, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (19, 0),
                span: Span(17, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (7, 0),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                children: (8, 9),
                span: Span(23, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                span: Span(27, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                children: (3, 1),
                span: Span(30, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (17, 0),
                span: Span(30, 45),
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
                children: (12, 0),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                children: (13, 14),
                span: Span(36, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (15, 0),
                span: Span(36, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (1, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (4, 1),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (22, 0),
                span: Span(47, 48),
            },
        ]
    );
}
