use crate::{
    resolver::{
        declarations::{Definition, Visibility},
        scopes::ScopeKind,
        types::TypeId,
    },
    source::{Source, SourceFile},
};

use super::bind_declarations;

#[test]
fn with_method_and_const() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed(
        "test",
        "trait Foo { fn bar(x: i64); const N: i64; }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (foo_id, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Trait {
        inner_scope_id,
        declarations,
        ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    assert_eq!(declarations.len(), 2);

    let member_ids = resolver
        .declarations
        .get_declaration_members(&declarations)
        .unwrap();

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let n_symbol = resolver.symbols.add_symbol("N");

    let bar_decl = resolver
        .declarations
        .get_declaration(member_ids[0])
        .unwrap();
    let Definition::Function {
        value_parameters,
        return_type_id,
        ..
    } = bar_decl.definition
    else {
        panic!();
    };

    assert_eq!(bar_decl.symbol_id, bar_symbol);
    assert_eq!(value_parameters.len(), 1);
    assert_eq!(return_type_id, TypeId::UNIT);
    assert_eq!(bar_decl.scope_id, inner_scope_id);

    let n_decl = resolver
        .declarations
        .get_declaration(member_ids[1])
        .unwrap();
    let Definition::AssociatedConstant {
        parent, type_id, ..
    } = n_decl.definition
    else {
        panic!();
    };

    assert_eq!(n_decl.symbol_id, n_symbol);
    assert_eq!(type_id, TypeId::I_64);
    assert_eq!(parent, foo_id);
    assert_eq!(n_decl.scope_id, inner_scope_id);
}

#[test]
fn supertraits_resolved() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed(
        "test",
        "trait Bar {} trait Foo: Bar {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (bar_id, _) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Trait { supertraits, .. } = foo_declaration.definition else {
        panic!();
    };

    assert_eq!(supertraits.len(), 1);

    let supertrait_ids = resolver
        .declarations
        .get_declaration_members(&supertraits)
        .unwrap();

    assert_eq!(supertrait_ids[0], bar_id);
}

#[test]
fn trait_creates_trait_scope() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("test", "trait Foo {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Trait { inner_scope_id, .. } = foo_declaration.definition else {
        panic!();
    };

    let scope = resolver.scopes.get_scope(inner_scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Trait);
    assert_eq!(scope.parent, crate_scope_id);
}
