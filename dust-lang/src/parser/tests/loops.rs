use crate::{
    parser::parse,
    parser::syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload},
    source::Span,
    tests::loop_cases,
};

#[test]
fn while_loop() {
    let source = loop_cases::WHILE_LOOP;
    let (syntax_tree, error) = parse(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                payload: SyntaxPayload::binary_children(SyntaxId(5), SyntaxId(3)),
                span: Span(0, 49),
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(1), SyntaxId(3)),
                span: Span(1, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(1)),
                span: Span(9, 10),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::empty(),
                span: Span(13, 14),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(3)),
                span: Span(13, 15),
            },
            SyntaxNode {
                kind: SyntaxKind::WhileExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(10), SyntaxId(17)),
                span: Span(17, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(18)),
                span: Span(17, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(7)),
                span: Span(23, 24),
            },
            SyntaxNode {
                kind: SyntaxKind::LessThanExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(8), SyntaxId(9)),
                span: Span(23, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(42),
                span: Span(27, 29),
            },
            SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                payload: SyntaxPayload::binary_children(SyntaxId(6), SyntaxId(1)),
                span: Span(30, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                payload: SyntaxPayload::child(SyntaxId(16)),
                span: Span(30, 45),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(8)),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(12)),
                span: Span(36, 37),
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentStatement,
                payload: SyntaxPayload::binary_children(SyntaxId(13), SyntaxId(14)),
                span: Span(36, 43),
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                payload: SyntaxPayload::encode_integer(1),
                span: Span(41, 42),
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                payload: SyntaxPayload::empty(),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                payload: SyntaxPayload::child(SyntaxId(10)),
                span: Span(47, 48),
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                payload: SyntaxPayload::child(SyntaxId(21)),
                span: Span(47, 48),
            },
        ]
    );
}
