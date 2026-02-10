use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::local_cases,
};

#[test]
fn local_boolean_not() {
    let source = local_cases::LOCAL_BOOLEAN_NOT;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(2)),
                span: Span(0, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(0), SyntaxId(3)),
                span: Span(1, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(5, 6),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                payload: SyntaxPayload::empty(),
                span: Span(8, 12),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(true),
                span: Span(15, 19),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(4)),
                span: Span(15, 20),
            },
            SyntaxNode {
                kind: SyntaxKind::NotExpression,
                payload: SyntaxPayload::child(SyntaxId(9)),
                span: Span(21, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(22, 23),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(22, 23),
            },
        ]
    );
}
