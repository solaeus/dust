use crate::{resolver::scope_graph::ScopeKind, source::SourceFileId, syntax::SyntaxKind};

use super::bind_declarations;

#[test]
fn body_is_in_function_scope() {
    let (_syntax, mut resolver) = bind_declarations("fn main() { let f = () { let y = 1; }; }");

    let y_symbol_id = resolver.symbols.add_symbol("y");
    let y_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == y_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let y_scope = resolver.scopes.get_scope(y_scope_id).unwrap();
    let block_scope = resolver.scopes.get_scope(y_scope.parent).unwrap();
    let function_scope = resolver.scopes.get_scope(block_scope.parent).unwrap();

    assert_eq!(function_scope.kind, ScopeKind::Function);
}

#[test]
fn body_binds_to_function_scope() {
    let (syntax, resolver) = bind_declarations("fn main() { let f = () { let y = 1; }; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let fn_exprs = tree
        .iter()
        .filter(|reader| reader.kind() == SyntaxKind::FunctionExpression)
        .collect::<Vec<_>>();
    let inner_fn = fn_exprs.last().unwrap();
    let (_, body) = inner_fn.binary_children().unwrap();

    let scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();
    let parent_scope = resolver.scopes.get_scope(scope.parent).unwrap();

    assert_eq!(parent_scope.kind, ScopeKind::Function);
}
