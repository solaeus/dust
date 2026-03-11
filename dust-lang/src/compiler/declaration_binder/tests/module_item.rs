use crate::{
    compiler::declaration_binder::tests::{bind_declarations, find_declaration},
    resolver::{
        declaration_graph::{Definition, ModuleKind},
        scope_graph::ScopeKind,
    },
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn declares_module() {
    let (syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();
    let foo_declaration = resolver.declarations.get_declaration(foo_id).unwrap();

    let inner_scope_id = match foo_declaration.definition {
        Definition::Module {
            kind: ModuleKind::Inline,
            inner_scope_id,
        } => inner_scope_id,
        other => panic!("expected inline Module declaration, got {other:?}"),
    };

    assert!(!foo_declaration.public);

    let inner_scope = resolver.scopes.get_scope(inner_scope_id).unwrap();

    assert_eq!(inner_scope.kind, ScopeKind::Module);

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .unwrap();

    assert_eq!(bar_declaration.scope_id, inner_scope_id);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let module = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ModuleItem)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&module.id).unwrap(),
        foo_id
    );

    let body = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ModuleBody)
        .unwrap();
    let body_scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let body_scope = resolver.scopes.get_scope(*body_scope_id).unwrap();

    assert_eq!(body_scope.kind, ScopeKind::Module);
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

    assert!(foo_declaration.public);
}
