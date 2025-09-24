use crate::{
    Span,
    parser::parse_main,
    syntax_tree::{SyntaxKind, SyntaxNode},
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
                children: (0, 2),
                span: Span(0, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                span: Span(1, 20),
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
                children: (2, 0),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::NotExpression,
                children: (5, 0),
                span: Span(21, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (3, 0),
                span: Span(22, 23),
            },
        ]
    );
}
