use crate::{
    resolver::{
        declarations::{Definition, Visibility},
        types::{Type, TypeId},
    },
    source::{Source, SourceFile},
};

use super::bind_declarations;

#[test]
fn simple() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("test", "type Foo = i64;"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::TypeAlias {
        public,
        type_parameters,
        aliased_type_id,
    } = foo_declaration.definition
    else {
        panic!();
    };

    assert!(!public);
    assert!(type_parameters.is_empty());
    assert_eq!(aliased_type_id, TypeId::I_64);
    assert_eq!(foo_declaration.scope_id, crate_scope_id);
}

#[test]
fn generic_alias_resolves_type_parameter() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("test", "type Pair<T> = T;"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let pair_symbol = resolver.symbols.add_symbol("Pair");
    let (_, pair_declaration) = resolver
        .declarations
        .find_declaration(pair_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::TypeAlias {
        type_parameters,
        aliased_type_id,
        ..
    } = pair_declaration.definition
    else {
        panic!();
    };

    assert_eq!(type_parameters.len(), 1);

    let type_parameter_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();

    let t_declaration_id = type_parameter_ids[0];
    let t_symbol = resolver.symbols.add_symbol("T");
    let t_declaration = resolver
        .declarations
        .get_declaration(t_declaration_id)
        .unwrap();

    assert_eq!(t_declaration.symbol_id, t_symbol);
    assert!(matches!(
        t_declaration.definition,
        Definition::TypeParameter
    ));

    let aliased_type = resolver.types.get_type(aliased_type_id).unwrap();
    let Type::Generic { declaration_id } = aliased_type else {
        panic!();
    };

    assert_eq!(*declaration_id, t_declaration_id);
}
