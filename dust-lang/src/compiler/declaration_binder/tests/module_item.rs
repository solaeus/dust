use crate::{
    resolver::{
        declaration_graph::{DeclarationKind, ModuleKind},
        scope_graph::ScopeKind,
    },
    source::SourceFileId,
    syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn declares_module() {
    let (_syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(matches!(
        foo_declaration.kind,
        DeclarationKind::Module {
            kind: ModuleKind::Inline,
            ..
        }
    ));
}

#[test]
fn creates_module_scope() {
    let (_syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    let inner_scope_id = match foo_declaration.kind {
        DeclarationKind::Module { inner_scope_id, .. } => inner_scope_id,
        other => panic!("expected Module declaration, got {other:?}"),
    };

    let inner_scope = resolver.scopes.get_scope(inner_scope_id).unwrap();

    assert_eq!(inner_scope.kind, ScopeKind::Module);
}

#[test]
fn items_declared_in_module_scope() {
    let (_syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    let module_scope_id = match foo_declaration.kind {
        DeclarationKind::Module { inner_scope_id, .. } => inner_scope_id,
        other => panic!("expected Module declaration, got {other:?}"),
    };

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .unwrap();

    assert_eq!(bar_declaration.scope_id, module_scope_id);
}

#[test]
fn binds_to_module_declaration() {
    let (syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");
    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let module = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ModuleItem)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&module.id).unwrap(),
        foo_id
    );
}

#[test]
fn body_binds_to_module_scope() {
    let (syntax, resolver) = bind_declarations("mod foo { fn bar() {} }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let body = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ModuleBody)
        .unwrap();

    let scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Module);
}

#[test]
fn is_not_public_by_default() {
    let (_syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

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
    let (_syntax, mut resolver) = bind_declarations("pub mod foo { fn bar() {} }");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(foo_declaration.is_public);
}
