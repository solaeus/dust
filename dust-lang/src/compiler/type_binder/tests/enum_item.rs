use crate::resolver::type_graph::{TypeId, TypeNode};

use super::{bind_types, find_declaration};

#[test]
fn creates_enum_type() {
    let (_syntax, mut resolver) = bind_types("enum Color { Red }");

    let (color_id, _) = find_declaration(&mut resolver, "Color").unwrap();
    let color_type_id = *resolver
        .declarations
        .get_declaration_type(&color_id)
        .unwrap();
    let color_type = *resolver.types.get_type(color_type_id).unwrap();

    assert!(matches!(color_type, TypeNode::Enum { .. }));
}

#[test]
fn variant_without_fields_has_unit_type() {
    let (_syntax, mut resolver) = bind_types("enum Color { Red }");

    let (red_id, _) = find_declaration(&mut resolver, "Red").unwrap();
    let red_type = *resolver.declarations.get_declaration_type(&red_id).unwrap();

    assert_eq!(red_type, TypeId::UNIT);
}
