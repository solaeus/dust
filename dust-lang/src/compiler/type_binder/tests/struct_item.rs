use crate::{
    compiler::type_binder::tests::{bind_types, find_declaration},
    resolver::type_graph::{Type, TypeId},
};

#[test]
fn creates_struct_type() {
    let (_syntax, mut resolver) = bind_types("struct Foo { x: i64 }");

    let (foo_id, _) = find_declaration(&mut resolver, "Foo").unwrap();
    let foo_type_id = *resolver.declarations.get_declaration_type(&foo_id).unwrap();
    let foo_type = *resolver.types.get_type(foo_type_id).unwrap();

    assert!(matches!(foo_type, Type::Struct { .. }));
}

#[test]
fn field_gets_declared_type() {
    let (_syntax, mut resolver) = bind_types("struct Foo { x: i64 }");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    assert_eq!(x_type, TypeId::I_64);
}

#[test]
fn multiple_fields_get_correct_types() {
    let (_syntax, mut resolver) = bind_types("struct Foo { x: i64, y: bool }");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    let (y_id, _) = find_declaration(&mut resolver, "y").unwrap();
    let y_type = *resolver.declarations.get_declaration_type(&y_id).unwrap();

    assert_eq!(x_type, TypeId::I_64);
    assert_eq!(y_type, TypeId::BOOLEAN);
}
