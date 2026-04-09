use crate::{
    compiler::resolver::{
        declarations::{Definition, Visibility},
        types::TypeId,
    },
    source::{Code, Source},
};

use super::bind_declarations;

#[test]
fn with_method() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "trait Bar { fn baz(); } struct Foo {} impl Bar for Foo { fn baz() {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (bar_id, _) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
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

    assert_eq!(trait_declaration_id, Some(bar_id));
    assert_eq!(declarations.len(), 1);

    let member_ids = resolver
        .declarations
        .get_declaration_members(&declarations)
        .unwrap();
    let baz_decl = resolver
        .declarations
        .get_declaration(member_ids[0])
        .unwrap();
    let baz_symbol = resolver.symbols.add_symbol("baz");

    assert_eq!(baz_decl.symbol_id, baz_symbol);
    assert!(matches!(baz_decl.definition, Definition::Function { .. }));
}

#[test]
fn with_associated_type() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
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

    assert_eq!(declarations.len(), 1);

    let member_ids = resolver
        .declarations
        .get_declaration_members(&declarations)
        .unwrap();
    let item_decl = resolver
        .declarations
        .get_declaration(member_ids[0])
        .unwrap();
    let Definition::AssociatedType {
        aliased_type_id, ..
    } = item_decl.definition
    else {
        panic!();
    };

    assert_eq!(aliased_type_id, TypeId::I_64);
}
