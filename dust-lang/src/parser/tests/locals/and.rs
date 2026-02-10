use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::local_cases,
};

#[test]
fn local_boolean_and() {
    let source = local_cases::LOCAL_BOOLEAN_AND;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(3)),
                span: Span(0, 49),
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
                kind: SyntaxKind::LetStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(3), SyntaxId(3)),
                span: Span(21, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(25, 26),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanType,
                payload: SyntaxPayload::empty(),
                span: Span(28, 32),
            },
            SyntaxNode {
                kind: SyntaxKind::BooleanExpression,
                payload: SyntaxPayload::encode_boolean(false),
                span: Span(35, 40),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(35, 41),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(13)),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(14)),
                span: Span(42, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::AndExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(15), SyntaxId(18)),
                span: Span(42, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(17)),
                span: Span(47, 48),
            },
        ]
    );
}
