use crate::{
    resolver::{declaration_graph::DeclarationKind, scope_graph::ScopeKind},
    source::SourceFileId,
    syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_function_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn foo() {}");

    let (foo_id, foo_kind) = find_declaration(&mut resolver, "foo").unwrap();

    assert_eq!(foo_kind, DeclarationKind::Function);

    let foo_declaration = resolver.declarations.get_declaration(foo_id).unwrap();

    assert!(!foo_declaration.is_public);

    let declaration_scope = resolver.scopes.get_scope(foo_declaration.scope_id).unwrap();

    assert_eq!(declaration_scope.kind, ScopeKind::Crate);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let func = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::FunctionItem)
        .unwrap();
    let (name, _) = func.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), foo_id);
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
