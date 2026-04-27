use crate::{
    compiler::{
        resolver::{declarations::Definition, types::TypeId},
        tests::bind_declarations,
    },
    source::{Source, SourceCode},
};

#[test]
fn with_method() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "trait Bar { fn baz(); } struct Foo {} impl Bar for Foo { fn baz() {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (bar_id, _) = resolver
        .declarations
        .find_declaration_id(bar_symbol, crate_scope_id)
        .unwrap();

    let (_, impl_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, d)| matches!(d.definition, Definition::TraitImplementation { .. }))
        .unwrap();
    let Definition::TraitImplementation {
        trait_declaration_id,
        declarations,
        ..
    } = impl_declaration.definition
    else {
        panic!();
    };

    assert_eq!(trait_declaration_id, bar_id);
    assert_eq!(resolver.scopes.namespace_len(declarations.unwrap()), 1);

    let member_entries = resolver.scopes.get_namespace_entries(declarations.unwrap());
    let baz_decl = resolver
        .declarations
        .get_declaration(member_entries[0].1)
        .unwrap();
    let baz_symbol = resolver.symbols.add_symbol("baz");

    assert_eq!(baz_decl.symbol_id, baz_symbol);
    assert!(matches!(baz_decl.definition, Definition::Function { .. }));
}

#[test]
fn with_associated_type() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "trait Bar { type Item; } struct Foo {} impl Bar for Foo { type Item = i64; }",
    ));

    let (_syntax, resolver, _crate_scope_id) = bind_declarations(&source);
    let (_, impl_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, d)| matches!(d.definition, Definition::TraitImplementation { .. }))
        .unwrap();
    let Definition::TraitImplementation { declarations, .. } = impl_declaration.definition else {
        panic!();
    };

    assert_eq!(resolver.scopes.namespace_len(declarations.unwrap()), 1);

    let member_entries = resolver.scopes.get_namespace_entries(declarations.unwrap());
    let item_decl = resolver
        .declarations
        .get_declaration(member_entries[0].1)
        .unwrap();
    let Definition::InherentAssociatedType {
        aliased_type_id, ..
    } = item_decl.definition
    else {
        panic!();
    };

    assert_eq!(aliased_type_id, TypeId::I_64);
}
