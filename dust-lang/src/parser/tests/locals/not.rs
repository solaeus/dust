use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_boolean_not() {
    let source = local_cases::LOCAL_BOOLEAN_NOT;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 24),
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
                kind: SyntaxKind::NotExpression,
                children: (5, 0),
                position: Span(21, 23),
                payload: TypeId::BOOLEAN.0
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (256, 0),
                position: Span(22, 23),
                payload: TypeId::BOOLEAN.0
            },
        ]
    );
}
