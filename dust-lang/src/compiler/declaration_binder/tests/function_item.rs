use crate::{
    resolver::{declaration_graph::DeclarationKind, scope_graph::ScopeKind},
    source::SourceFileId,
    syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_function_declaration() {
    let (_syntax, mut resolver) = bind_declarations("fn foo() {}");

    let (_, foo_kind) = find_declaration(&mut resolver, "foo").unwrap();

    assert_eq!(foo_kind, DeclarationKind::Function);
}

#[test]
fn is_not_public_by_default() {
    let (_syntax, mut resolver) = bind_declarations("fn foo() {}");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(!foo_declaration.is_public);
}

#[test]
fn is_public_when_pub() {
    let (_syntax, mut resolver) = bind_declarations("pub fn foo() {}");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(foo_declaration.is_public);
}

#[test]
fn declares_in_crate_scope() {
    let (_syntax, mut resolver) = bind_declarations("fn foo() {}");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    let declaration_scope = resolver.scopes.get_scope(foo_declaration.scope_id).unwrap();

    assert_eq!(declaration_scope.kind, ScopeKind::Crate);
}

#[test]
fn binds_name_to_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn foo() {}");
    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let func = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::FunctionItem)
        .unwrap();
    let (name, _) = func.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), foo_id);
}

#[test]
fn parameters_create_local_declarations() {
    let (_syntax, mut resolver) = bind_declarations("fn foo(x: int, y: bool) {}");

    let (_, x_kind) = find_declaration(&mut resolver, "x").unwrap();
    let (_, y_kind) = find_declaration(&mut resolver, "y").unwrap();

    assert!(matches!(x_kind, DeclarationKind::Local));
    assert!(matches!(y_kind, DeclarationKind::Local));
}

#[test]
fn parameter_binds_to_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn foo(x: int) {}");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

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
