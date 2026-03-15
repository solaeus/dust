use crate::{
    compiler::declaration_binder::tests::{bind_declarations, find_declaration},
    resolver::scopes::ScopeKind,
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn body_creates_block_scope() {
    let (syntax, mut resolver) = bind_declarations("fn main() { while true { let x = 1; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let while_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::WhileExpression)
        .unwrap();
    let (_condition, body) = while_expr.binary_children().unwrap();

    let scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Block);

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_declaration = resolver.declarations.get_declaration(x_id).unwrap();
    let x_scope = resolver.scopes.get_scope(x_declaration.scope_id).unwrap();

    assert_eq!(x_scope.kind, ScopeKind::Block);
}
