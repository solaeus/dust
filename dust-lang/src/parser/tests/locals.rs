use crate::{
    Span,
    parser::parse_main,
    resolver::TypeId,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_declaration() {
    let source = local_cases::LOCAL_DECLARATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 16),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(0, 16),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(7, 10),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(13, 15),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(13, 16),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn local_mut_declaration() {
    let source = local_cases::LOCAL_MUT_DECLARATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 1),
                position: Span(0, 20),
                payload: TypeId::NONE.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (1, 3),
                position: Span(0, 20),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(11, 14),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(17, 19),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(17, 20),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}

#[test]
fn local_evaluation() {
    let source = local_cases::LOCAL_EVALUATION;
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (0, 2),
                position: Span(0, 18),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (1, 3),
                position: Span(0, 16),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                position: Span(7, 10),
                payload: 0,
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (42, 0),
                position: Span(13, 15),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (2, 0),
                position: Span(13, 16),
                payload: TypeId::INTEGER.0,
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (0, 0),
                position: Span(17, 18),
                payload: TypeId::INTEGER.0,
            }
        ]
    );
}
