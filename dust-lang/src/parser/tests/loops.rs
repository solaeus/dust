use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
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
                payload: (8, 3),
                span: Span(0, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: (1, 3),
                span: Span(1, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (0, 1),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (0, 0),
                span: Span(13, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (3, 0),
                span: Span(13, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::WhileExpression,
                payload: (10, 17),
                span: Span(17, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (18, 0),
                span: Span(17, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (4, 1),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (7, 0),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: (8, 9),
                span: Span(23, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (42, 0),
                span: Span(27, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: (6, 1),
                span: Span(30, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (16, 0),
                span: Span(30, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (5, 1),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (12, 0),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                payload: (13, 14),
                span: Span(36, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: (1, 0),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: (0, 0),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: (7, 1),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: (21, 0),
                span: Span(47, 48),
            },
        ]
    );
}
