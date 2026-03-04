use crate::{resolver::scope_graph::ScopeKind, source::SourceFileId, syntax::SyntaxKind};

use super::bind_declarations;

#[test]
fn body_creates_block_scope() {
    let (syntax, resolver) = bind_declarations("fn main() { while true { let x = 1; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let while_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::WhileExpression)
        .unwrap();
    let (_condition, body) = while_expr.binary_children().unwrap();

    let scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Block);
}
