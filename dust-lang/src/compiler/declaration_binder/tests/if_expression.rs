use crate::{
    compiler::declaration_binder::tests::bind_declarations, resolver::scope_graph::ScopeKind,
    source::SourceFileId, syntax::node::SyntaxKind,
};

#[test]
fn creates_scopes_for_both_branches() {
    let (syntax, resolver) =
        bind_declarations("fn main() { if true { let a = 1; } else { let b = 2; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let if_expression = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::IfExpression)
        .unwrap();
    let mut children = if_expression.children();
    let _condition = children.next().unwrap();
    let then_block = children.next().unwrap();
    let else_block = children.next().unwrap();

    let then_scope_id = resolver.get_scope_binding(&then_block.id).unwrap();
    let then_scope = resolver.scopes.get_scope(*then_scope_id).unwrap();
    assert_eq!(then_scope.kind, ScopeKind::Block);

    let else_scope_id = resolver.get_scope_binding(&else_block.id).unwrap();
    let else_scope = resolver.scopes.get_scope(*else_scope_id).unwrap();
    assert_eq!(else_scope.kind, ScopeKind::Block);

    assert_ne!(then_scope_id, else_scope_id);
}
