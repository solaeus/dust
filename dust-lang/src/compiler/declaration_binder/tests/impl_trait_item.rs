use crate::{
    compiler::{
        resolver::{declarations::Definition, types::TypeId},
        tests::bind_declarations,
    },
    source::{Source, SourceCode, SourceCodeId},
    syntax::node::SyntaxKind,
};

#[test]
fn with_method() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "trait Bar { fn baz(); } struct Foo {} impl Bar for Foo { fn baz() {} }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let bar_declaration_id = *resolver
        .declarations
        .find_declaration_id(bar_symbol, crate_scope_id)
        .unwrap();

    let tree = syntax.get_tree(SourceCodeId::MAIN).unwrap();
    let implementation_item = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::ImplItem)
        .unwrap();
    let implementation_declaration_id = *resolver
        .get_declaration_binding(&implementation_item.id)
        .unwrap();
    let implementation_declaration = resolver
        .declarations
        .get_declaration(implementation_declaration_id)
        .unwrap();
    let Definition::TraitImplementation {
        trait_declaration_id,
        declarations,
        ..
    } = implementation_declaration.definition
    else {
        panic!();
    };

    assert_eq!(trait_declaration_id, bar_declaration_id);
    assert_eq!(resolver.scopes.get_members(declarations.unwrap()).len(), 1);

    let member_entries = resolver.scopes.get_members(declarations.unwrap());
    let baz_declaration = resolver
        .declarations
        .get_declaration(member_entries[0])
        .unwrap();
    let baz_symbol = resolver.symbols.add_symbol("baz");

    assert_eq!(baz_declaration.symbol_id, baz_symbol);
    assert!(matches!(
        baz_declaration.definition,
        Definition::Function { .. }
    ));
}

#[test]
fn with_associated_type() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "trait Bar { type Item; } struct Foo {} impl Bar for Foo { type Item = i64; }",
    ));

    let (syntax, resolver, _crate_scope_id) = bind_declarations(&source);
    let tree = syntax.get_tree(SourceCodeId::MAIN).unwrap();
    let implementation_item = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::ImplItem)
        .unwrap();
    let implementation_declaration_id = *resolver
        .get_declaration_binding(&implementation_item.id)
        .unwrap();
    let implementation_declaration = resolver
        .declarations
        .get_declaration(implementation_declaration_id)
        .unwrap();
    let Definition::TraitImplementation { declarations, .. } =
        implementation_declaration.definition
    else {
        panic!();
    };

    assert_eq!(resolver.scopes.get_members(declarations.unwrap()).len(), 1);

    let member_entries = resolver.scopes.get_members(declarations.unwrap());
    let item_declaration = resolver
        .declarations
        .get_declaration(member_entries[0])
        .unwrap();
    let Definition::InherentAssociatedType {
        aliased_type_id, ..
    } = item_declaration.definition
    else {
        panic!();
    };

    assert_eq!(aliased_type_id, TypeId::I_64);
}
