use crate::{
    resolver::{declaration_graph::DeclarationKind, scope_graph::ScopeKind},
    source::SourceFileId,
    syntax::SyntaxKind,
};

use super::bind_declarations;

#[test]
fn creates_declaration() {
    let (syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } use foo::bar; fn main() { bar() }");

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let bar_declarations: Vec<_> = resolver
        .declarations
        .iter()
        .filter(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .collect();

    let use_declaration = bar_declarations
        .iter()
        .find(|(_, declaration)| {
            let scope = resolver.scopes.get_scope(declaration.scope_id).unwrap();
            scope.kind == ScopeKind::Crate
        })
        .expect("expected a bar declaration in crate scope");

    assert!(matches!(
        use_declaration.1.kind,
        DeclarationKind::Function { .. }
    ));
    assert!(!use_declaration.1.public);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let use_item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::UseItem)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&use_item.id).unwrap(),
        use_declaration.0
    );
}

#[test]
fn creates_distinct_declaration() {
    let (_syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } use foo::bar; fn main() { bar() }");

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let bar_declarations: Vec<_> = resolver
        .declarations
        .iter()
        .filter(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .map(|(id, _)| id)
        .collect();

    assert_eq!(
        bar_declarations.len(),
        2,
        "expected two distinct bar declarations (original and use-imported)"
    );
    assert_ne!(
        bar_declarations[0], bar_declarations[1],
        "use-imported declaration must have a different DeclarationId from the original"
    );
}

#[test]
fn is_public_when_pub() {
    let (_syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } pub use foo::bar; fn main() { bar() }");

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let use_declaration = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| {
            declaration.symbol_id == bar_symbol_id && {
                let scope = resolver.scopes.get_scope(declaration.scope_id).unwrap();
                scope.kind == ScopeKind::Crate
            }
        })
        .expect("expected a bar declaration in crate scope");

    assert!(use_declaration.1.public);
}
