use crate::resolver::type_graph::{TypeId, TypeNode};

use super::{bind_types, find_declaration};

#[test]
fn creates_function_type() {
    let (_syntax, mut resolver) = bind_types("fn foo() {}");

    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();
    let foo_type_id = *resolver.declarations.get_declaration_type(&foo_id).unwrap();
    let foo_type = *resolver.types.get_type(foo_type_id).unwrap();

    assert!(matches!(foo_type, TypeNode::Function { .. }));
}

#[test]
fn return_type_matches_annotation() {
    let (_syntax, mut resolver) = bind_types("fn foo() -> i64 { 42 }");

    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();
    let foo_type_id = *resolver.declarations.get_declaration_type(&foo_id).unwrap();
    let foo_type = *resolver.types.get_type(foo_type_id).unwrap();

    let return_type_id = match foo_type {
        TypeNode::Function { return_type_id, .. } => return_type_id,
        other => panic!("expected Function type, got {other:?}"),
    };

    assert_eq!(return_type_id, TypeId::I_64);
}

#[test]
fn parameters_get_declared_types() {
    let (_syntax, mut resolver) = bind_types("fn foo(x: i64, y: bool) {}");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    let (y_id, _) = find_declaration(&mut resolver, "y").unwrap();
    let y_type = *resolver.declarations.get_declaration_type(&y_id).unwrap();

    assert_eq!(x_type, TypeId::I_64);
    assert_eq!(y_type, TypeId::BOOLEAN);
}

#[test]
fn without_return_type_has_unit_return() {
    let (_syntax, mut resolver) = bind_types("fn foo() {}");

    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();
    let foo_type_id = *resolver.declarations.get_declaration_type(&foo_id).unwrap();
    let foo_type = *resolver.types.get_type(foo_type_id).unwrap();

    let return_type_id = match foo_type {
        TypeNode::Function { return_type_id, .. } => return_type_id,
        other => panic!("expected Function type, got {other:?}"),
    };

    assert_eq!(return_type_id, TypeId::UNIT);
}
