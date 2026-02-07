use crate::{
    parser::parse_main,
    source::Span,
    syntax::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean_not() {
    let source = local_cases::LOCAL_BOOLEAN_NOT.to_string();
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
                payload: (1, 0),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: (4, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::NotExpression,
                payload: (9, 0),
                span: Span(21, 23),
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
