use crate::{
    compiler::declaration_binder::tests::{bind_declarations, find_declaration},
    resolver::{declarations::Definition, scopes::ScopeKind},
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn creates_function_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn foo() {}");

    let (foo_id, foo_kind) = find_declaration(&mut resolver, "foo").unwrap();

    assert!(matches!(foo_kind, Definition::Function { .. }));

    let foo_declaration = resolver.declarations.get_declaration(foo_id).unwrap();

    assert!(!foo_declaration.public);

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

    assert!(foo_declaration.public);
}
