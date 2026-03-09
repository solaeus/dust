use crate::{
    resolver::{declaration_graph::DeclarationKind, scope_graph::ScopeKind},
    source::SourceFileId,
    syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_function_scope_and_binds_body() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let f = fn () { let y = 1; }; }");

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

    let y_symbol_id = resolver.symbols.add_symbol("y");
    let y_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == y_symbol_id)
        .unwrap()
        .1
        .scope_id;
    let y_scope = resolver.scopes.get_scope(y_scope_id).unwrap();
    let y_parent_scope = resolver.scopes.get_scope(y_scope.parent).unwrap();

    assert_eq!(y_parent_scope.kind, ScopeKind::Function);
}

#[test]
fn parameters_are_local_in_function_scope() {
    let (syntax, mut resolver) = bind_declarations("fn foo(x: i64, y: bool) {}");

    let (x_id, x_kind) = find_declaration(&mut resolver, "x").unwrap();
    let (_, y_kind) = find_declaration(&mut resolver, "y").unwrap();

    assert!(matches!(x_kind, DeclarationKind::Local { .. }));
    assert!(matches!(y_kind, DeclarationKind::Local { .. }));

    let x_declaration = resolver.declarations.get_declaration(x_id).unwrap();
    let x_scope = resolver.scopes.get_scope(x_declaration.scope_id).unwrap();
    assert_eq!(x_scope.kind, ScopeKind::Function);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let value_params = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ValueParameters)
        .unwrap();
    let param_name = value_params.children().unwrap().next().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&param_name.id).unwrap(),
        x_id
    );
}
