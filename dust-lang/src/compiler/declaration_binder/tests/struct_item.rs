use crate::{
    resolver::{
        declarations::{Definition, Visibility},
        types::TypeId,
    },
    source::{Source, SourceFile},
};

use super::bind_declarations;

#[test]
fn empty_struct() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", "struct Foo {}"));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::StructType {
        public,
        type_parameters,
        fields,
    } = foo_declaration.definition
    else {
        panic!();
    };

    assert!(!public);
    assert!(type_parameters.is_empty());
    assert!(fields.is_empty());
    assert_eq!(foo_declaration.scope_id, crate_scope_id);
}

#[test]
fn struct_with_named_fields() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "struct Foo { x: i64, y: bool }",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (foo_id, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::StructType { fields, .. } = foo_declaration.definition else {
        panic!();
    };

    let field_ids = resolver
        .declarations
        .get_declaration_members(&fields)
        .unwrap();
    assert_eq!(field_ids.len(), 2);

    let x_symbol = resolver.symbols.add_symbol("x");
    let y_symbol = resolver.symbols.add_symbol("y");

    let first_field = resolver
        .declarations
        .get_declaration(field_ids[0])
        .unwrap();
    let Definition::Field {
        parent_struct: first_parent,
        type_id: first_type,
        ..
    } = first_field.definition
    else {
        panic!();
    };
    assert_eq!(first_field.symbol_id, x_symbol);
    assert_eq!(first_type, TypeId::I_64);
    assert_eq!(first_parent, foo_id);

    let second_field = resolver
        .declarations
        .get_declaration(field_ids[1])
        .unwrap();
    let Definition::Field {
        parent_struct: second_parent,
        type_id: second_type,
        ..
    } = second_field.definition
    else {
        panic!();
    };
    assert_eq!(second_field.symbol_id, y_symbol);
    assert_eq!(second_type, TypeId::BOOLEAN);
    assert_eq!(second_parent, foo_id);
}

#[test]
fn tuple_struct() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "struct Bar(i64, bool);",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (bar_id, bar_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::StructType { fields, .. } = bar_declaration.definition else {
        panic!();
    };

    let field_ids = resolver
        .declarations
        .get_declaration_members(&fields)
        .unwrap();
    assert_eq!(field_ids.len(), 2);

    let first_field = resolver
        .declarations
        .get_declaration(field_ids[0])
        .unwrap();
    let Definition::Field {
        parent_struct: first_parent,
        type_id: first_type,
        ..
    } = first_field.definition
    else {
        panic!();
    };
    assert_eq!(first_type, TypeId::I_64);
    assert_eq!(first_parent, bar_id);

    let second_field = resolver
        .declarations
        .get_declaration(field_ids[1])
        .unwrap();
    let Definition::Field {
        parent_struct: second_parent,
        type_id: second_type,
        ..
    } = second_field.definition
    else {
        panic!();
    };
    assert_eq!(second_type, TypeId::BOOLEAN);
    assert_eq!(second_parent, bar_id);
}

#[test]
fn public_generic_struct() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "pub struct Pair<A, B> { x: i64 }",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
    let pair_symbol = resolver.symbols.add_symbol("Pair");
    let (_, pair_declaration) = resolver
        .declarations
        .find_declaration(pair_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::StructType {
        public,
        type_parameters,
        fields,
    } = pair_declaration.definition
    else {
        panic!();
    };

    assert!(public);

    let type_param_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();
    assert_eq!(type_param_ids.len(), 2);

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");
    let first_tp = resolver
        .declarations
        .get_declaration(type_param_ids[0])
        .unwrap();
    let second_tp = resolver
        .declarations
        .get_declaration(type_param_ids[1])
        .unwrap();
    assert_eq!(first_tp.symbol_id, a_symbol);
    assert!(matches!(first_tp.definition, Definition::TypeParameter));
    assert_eq!(second_tp.symbol_id, b_symbol);
    assert!(matches!(second_tp.definition, Definition::TypeParameter));

    let field_ids = resolver
        .declarations
        .get_declaration_members(&fields)
        .unwrap();
    assert_eq!(field_ids.len(), 1);

    let field = resolver
        .declarations
        .get_declaration(field_ids[0])
        .unwrap();
    let Definition::Field { type_id, .. } = field.definition else {
        panic!();
    };
    assert_eq!(type_id, TypeId::I_64);
}

#[test]
fn fields_not_visible_at_module_scope() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "struct Foo { x: i64 }",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
    let x_symbol = resolver.symbols.add_symbol("x");
    let result =
        resolver
            .declarations
            .find_declaration(x_symbol, crate_scope_id, Visibility::Module);

    assert!(
        result.is_none(),
        "struct fields should not be visible at module scope"
    );
}

#[test]
fn same_name_struct_in_different_modules() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod a { struct Foo { x: i64 } } mod b { struct Foo { x: bool } }",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let a_symbol = resolver.symbols.add_symbol("a");
    let (_, a_decl) = resolver
        .declarations
        .find_declaration(a_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: a_scope,
        ..
    } = a_decl.definition
    else {
        panic!();
    };

    let b_symbol = resolver.symbols.add_symbol("b");
    let (_, b_decl) = resolver
        .declarations
        .find_declaration(b_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: b_scope,
        ..
    } = b_decl.definition
    else {
        panic!();
    };

    let foo_symbol = resolver.symbols.add_symbol("Foo");

    let (a_foo_id, a_foo) = resolver
        .declarations
        .find_declaration(foo_symbol, a_scope, Visibility::Module)
        .unwrap();
    let Definition::StructType {
        fields: a_fields, ..
    } = a_foo.definition
    else {
        panic!();
    };
    let a_field_ids = resolver
        .declarations
        .get_declaration_members(&a_fields)
        .unwrap();
    let a_field = resolver
        .declarations
        .get_declaration(a_field_ids[0])
        .unwrap();
    let Definition::Field {
        type_id: a_field_type,
        ..
    } = a_field.definition
    else {
        panic!();
    };
    assert_eq!(a_field_type, TypeId::I_64);

    let (b_foo_id, b_foo) = resolver
        .declarations
        .find_declaration(foo_symbol, b_scope, Visibility::Module)
        .unwrap();
    let Definition::StructType {
        fields: b_fields, ..
    } = b_foo.definition
    else {
        panic!();
    };
    let b_field_ids = resolver
        .declarations
        .get_declaration_members(&b_fields)
        .unwrap();
    let b_field = resolver
        .declarations
        .get_declaration(b_field_ids[0])
        .unwrap();
    let Definition::Field {
        type_id: b_field_type,
        ..
    } = b_field.definition
    else {
        panic!();
    };
    assert_eq!(b_field_type, TypeId::BOOLEAN);

    assert_ne!(a_foo_id, b_foo_id);
}
