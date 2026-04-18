use crate::{
    compiler::resolver::{
        declarations::{Definition, Visibility},
        scopes::ScopeId,
        types::{Type, TypeId},
    },
    source::{Code, Source},
};

use super::bind_declarations;

#[test]
fn with_unit_variants() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "enum Color { Red, Green, Blue }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let color_symbol = resolver.symbols.add_symbol("Color");
    let (color_id, color_declaration) = resolver
        .declarations
        .find_declaration(color_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::EnumType {
        public,
        type_parameters,
        variants,
    } = color_declaration.definition
    else {
        panic!();
    };

    assert!(!public);
    assert!(type_parameters == ScopeId::NONE);
    assert_eq!(color_declaration.scope_id, crate_scope_id);

    let variant_entries = resolver.scopes.get_namespace_entries(variants);
    assert_eq!(variant_entries.len(), 3);

    let red_symbol = resolver.symbols.add_symbol("Red");
    let green_symbol = resolver.symbols.add_symbol("Green");
    let blue_symbol = resolver.symbols.add_symbol("Blue");

    let red = resolver
        .declarations
        .get_declaration(variant_entries[0].1)
        .unwrap();
    let Definition::Variant {
        discriminant: red_discriminant,
        enum_declaration_id: red_parent_id,
        ..
    } = red.definition
    else {
        panic!();
    };

    assert_eq!(red.symbol_id, red_symbol);
    assert_eq!(red_discriminant, 0);
    assert_eq!(red_parent_id, color_id);

    let green = resolver
        .declarations
        .get_declaration(variant_entries[1].1)
        .unwrap();
    let Definition::Variant {
        discriminant: green_discriminant,
        enum_declaration_id: green_parent_id,
        ..
    } = green.definition
    else {
        panic!();
    };

    assert_eq!(green.symbol_id, green_symbol);
    assert_eq!(green_discriminant, 1);
    assert_eq!(green_parent_id, color_id);

    let blue = resolver
        .declarations
        .get_declaration(variant_entries[2].1)
        .unwrap();
    let Definition::Variant {
        discriminant: blue_discriminant,
        enum_declaration_id: blue_parent_id,
        ..
    } = blue.definition
    else {
        panic!();
    };

    assert_eq!(blue.symbol_id, blue_symbol);
    assert_eq!(blue_discriminant, 2);
    assert_eq!(blue_parent_id, color_id);
}

#[test]
fn public_generic() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "pub enum Opt<A, B> { None, Some }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let opt_symbol = resolver.symbols.add_symbol("Opt");
    let (_, opt_declaration) = resolver
        .declarations
        .find_declaration(opt_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::EnumType {
        public,
        type_parameters,
        variants,
    } = opt_declaration.definition
    else {
        panic!();
    };

    assert!(public);

    let type_parameter_entries = resolver.scopes.get_namespace_entries(type_parameters);

    assert_eq!(type_parameter_entries.len(), 2);

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");
    let first_tp = resolver
        .declarations
        .get_declaration(type_parameter_entries[0].1)
        .unwrap();
    let second_tp = resolver
        .declarations
        .get_declaration(type_parameter_entries[1].1)
        .unwrap();

    assert_eq!(first_tp.symbol_id, a_symbol);
    assert!(matches!(first_tp.definition, Definition::TypeParameter));
    assert_eq!(second_tp.symbol_id, b_symbol);
    assert!(matches!(second_tp.definition, Definition::TypeParameter));
    assert_eq!(resolver.scopes.namespace_len(variants), 2);
}

#[test]
fn with_mixed_variants() {
    let mut source = Source::new();
    source.add_code(Code::validated_borrowed(
        "test",
        "enum Shape { Point, Line(i64), Rect { w: i64, h: i64 } }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let shape_symbol = resolver.symbols.add_symbol("Shape");
    let (shape_id, shape_declaration) = resolver
        .declarations
        .find_declaration(shape_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::EnumType { variants, .. } = shape_declaration.definition else {
        panic!();
    };

    let variant_entries = resolver.scopes.get_namespace_entries(variants);

    assert_eq!(variant_entries.len(), 3);

    let point_symbol = resolver.symbols.add_symbol("Point");
    let line_symbol = resolver.symbols.add_symbol("Line");
    let rect_symbol = resolver.symbols.add_symbol("Rect");

    let point = resolver
        .declarations
        .get_declaration(variant_entries[0].1)
        .unwrap();
    let Definition::Variant {
        discriminant: point_discriminant,
        enum_declaration_id: point_parent_id,
        fields: point_fields,
        ..
    } = point.definition
    else {
        panic!();
    };

    assert_eq!(point.symbol_id, point_symbol);
    assert_eq!(point_discriminant, 0);
    assert_eq!(point_parent_id, shape_id);
    assert!(point_fields == ScopeId::NONE);

    let line = resolver
        .declarations
        .get_declaration(variant_entries[1].1)
        .unwrap();
    let Definition::Variant {
        discriminant: line_discriminant,
        enum_declaration_id: line_parent_id,
        fields: line_fields,
        ..
    } = line.definition
    else {
        panic!();
    };

    assert_eq!(line.symbol_id, line_symbol);
    assert_eq!(line_discriminant, 1);
    assert_eq!(line_parent_id, shape_id);

    let line_field_entries = resolver.scopes.get_namespace_entries(line_fields);

    assert_eq!(line_field_entries.len(), 1);

    let line_field = resolver
        .declarations
        .get_declaration(line_field_entries[0].1)
        .unwrap();
    let Definition::Field {
        type_id: line_field_type,
        ..
    } = line_field.definition
    else {
        panic!();
    };

    assert_eq!(line_field_type, TypeId::I_64);

    let rect = resolver
        .declarations
        .get_declaration(variant_entries[2].1)
        .unwrap();
    let Definition::Variant {
        discriminant: rectangle_discriminant,
        enum_declaration_id: rectangle_parent_id,
        fields: rectangle_fields,
        ..
    } = rect.definition
    else {
        panic!();
    };

    assert_eq!(rect.symbol_id, rect_symbol);
    assert_eq!(rectangle_discriminant, 2);
    assert_eq!(rectangle_parent_id, shape_id);

    let rect_field_entries = resolver.scopes.get_namespace_entries(rectangle_fields);

    assert_eq!(rect_field_entries.len(), 2);

    let w_symbol = resolver.symbols.add_symbol("w");
    let h_symbol = resolver.symbols.add_symbol("h");
    let w_field = resolver
        .declarations
        .get_declaration(rect_field_entries[0].1)
        .unwrap();
    let h_field = resolver
        .declarations
        .get_declaration(rect_field_entries[1].1)
        .unwrap();

    assert_eq!(w_field.symbol_id, w_symbol);
    assert_eq!(h_field.symbol_id, h_symbol);
}

#[test]
fn variants_not_visible_at_module_scope() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed("test", "enum Foo { Bar }"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let result =
        resolver
            .declarations
            .find_declaration(bar_symbol, crate_scope_id, Visibility::Module);

    assert!(
        result.is_none(),
        "enum variants should not be visible at module scope"
    );
}

#[test]
fn same_name_in_different_modules() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "mod a { enum Foo { X } } mod b { enum Foo { Y } }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    let a_symbol = resolver.symbols.add_symbol("a");
    let (_, a_declaration) = resolver
        .declarations
        .find_declaration(a_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: a_scope,
        ..
    } = a_declaration.definition
    else {
        panic!();
    };

    let b_symbol = resolver.symbols.add_symbol("b");
    let (_, b_declaration) = resolver
        .declarations
        .find_declaration(b_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: b_scope,
        ..
    } = b_declaration.definition
    else {
        panic!();
    };

    let foo_symbol = resolver.symbols.add_symbol("Foo");

    let (a_foo_id, a_foo) = resolver
        .declarations
        .find_declaration(foo_symbol, a_scope, Visibility::Module)
        .unwrap();
    let Definition::EnumType {
        variants: a_variants,
        ..
    } = a_foo.definition
    else {
        panic!();
    };
    let a_variant_entries = resolver.scopes.get_namespace_entries(a_variants);
    let a_variant = resolver
        .declarations
        .get_declaration(a_variant_entries[0].1)
        .unwrap();
    let x_symbol = resolver.symbols.add_symbol("X");

    assert_eq!(a_variant.symbol_id, x_symbol);

    let (b_foo_id, b_foo) = resolver
        .declarations
        .find_declaration(foo_symbol, b_scope, Visibility::Module)
        .unwrap();
    let Definition::EnumType {
        variants: b_variants,
        ..
    } = b_foo.definition
    else {
        panic!();
    };
    let b_variant_entries = resolver.scopes.get_namespace_entries(b_variants);
    let b_variant = resolver
        .declarations
        .get_declaration(b_variant_entries[0].1)
        .unwrap();
    let y_symbol = resolver.symbols.add_symbol("Y");

    assert_eq!(b_variant.symbol_id, y_symbol);
    assert_ne!(a_foo_id, b_foo_id);
}

#[test]
fn generic_variant_field() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "enum Opt<T> { Some(T), None }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let opt_symbol = resolver.symbols.add_symbol("Opt");
    let (_, opt_declaration) = resolver
        .declarations
        .find_declaration(opt_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::EnumType {
        type_parameters,
        variants,
        ..
    } = opt_declaration.definition
    else {
        panic!();
    };

    let type_parameter_entries = resolver.scopes.get_namespace_entries(type_parameters);

    assert_eq!(type_parameter_entries.len(), 1);

    let t_declaration_id = type_parameter_entries[0].1;
    let variant_entries = resolver.scopes.get_namespace_entries(variants);

    assert_eq!(variant_entries.len(), 2);

    let some_variant = resolver
        .declarations
        .get_declaration(variant_entries[0].1)
        .unwrap();
    let Definition::Variant {
        fields: some_fields,
        ..
    } = some_variant.definition
    else {
        panic!();
    };
    let some_field_entries = resolver.scopes.get_namespace_entries(some_fields);

    assert_eq!(some_field_entries.len(), 1);

    let some_field = resolver
        .declarations
        .get_declaration(some_field_entries[0].1)
        .unwrap();
    let Definition::Field { type_id, .. } = some_field.definition else {
        panic!();
    };
    let field_type = resolver.types.get_type(type_id).unwrap();
    let Type::Generic { declaration_id } = field_type else {
        panic!();
    };

    assert_eq!(*declaration_id, t_declaration_id);

    let none_variant = resolver
        .declarations
        .get_declaration(variant_entries[1].1)
        .unwrap();
    let Definition::Variant {
        fields: none_fields,
        ..
    } = none_variant.definition
    else {
        panic!();
    };

    assert!(none_fields == ScopeId::NONE);
}
