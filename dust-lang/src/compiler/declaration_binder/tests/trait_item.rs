use crate::{
    compiler::{
        resolver::{declarations::Definition, scopes::ScopeKind, types::TypeId},
        tests::bind_declarations,
    },
    source::{Source, SourceCode},
};

#[test]
fn with_method_and_const() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "trait Foo { fn bar(x: i64); const N: i64; }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (foo_id, foo_declaration) = resolver
        .declarations
        .find_declaration_id(foo_symbol, crate_scope_id)
        .unwrap();
    let Definition::Trait { declarations, .. } = foo_declaration.definition else {
        panic!();
    };

    assert_eq!(resolver.scopes.namespace_len(declarations.unwrap()), 2);

    let member_entries = resolver.scopes.get_namespace_entries(declarations.unwrap());

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let n_symbol = resolver.symbols.add_symbol("N");

    let bar_decl = resolver
        .declarations
        .get_declaration(member_entries[0].1)
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
    assert_eq!(resolver.scopes.namespace_len(value_parameters.unwrap()), 1);
    assert_eq!(return_type_id, TypeId::UNIT);
    assert_eq!(bar_decl.scope_id, declarations.unwrap());

    let n_decl = resolver
        .declarations
        .get_declaration(member_entries[1].1)
        .unwrap();
    let Definition::InherentAssociatedConstant {
        parent, type_id, ..
    } = n_decl.definition
    else {
        panic!();
    };

    assert_eq!(n_decl.symbol_id, n_symbol);
    assert_eq!(type_id, TypeId::I_64);
    assert_eq!(parent, foo_id);
    assert_eq!(n_decl.scope_id, declarations.unwrap());
}

#[test]
fn supertraits_resolved() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "trait Bar {} trait Foo: Bar {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (bar_id, _) = resolver
        .declarations
        .find_declaration_id(bar_symbol, crate_scope_id)
        .unwrap();

    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration_id(foo_symbol, crate_scope_id)
        .unwrap();
    let Definition::Trait { supertraits, .. } = foo_declaration.definition else {
        panic!();
    };

    assert_eq!(resolver.scopes.namespace_len(supertraits.unwrap()), 1);

    let supertrait_entries = resolver.scopes.get_namespace_entries(supertraits.unwrap());

    assert_eq!(supertrait_entries[0].1, bar_id);
}

#[test]
fn trait_creates_trait_scope() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed("test", "trait Foo {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration_id(foo_symbol, crate_scope_id)
        .unwrap();
    let Definition::Trait { declarations, .. } = foo_declaration.definition else {
        panic!();
    };

    let scope = resolver.scopes.get_scope(declarations.unwrap());

    assert_eq!(scope.kind, ScopeKind::Members);
    assert_eq!(scope.parent, Some(crate_scope_id));
}
