use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::{bind_types, find_declaration};

#[test]
fn has_unit_type() {
    let (syntax, resolver) = bind_types("fn main() { let x = 42; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::LetStatement)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::UNIT);
}

#[test]
fn declaration_gets_expression_type() {
    let (_syntax, mut resolver) = bind_types("fn main() { let x = 42; }");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    assert_eq!(x_type, TypeId::INTEGER);
}

#[test]
fn type_annotation_sets_declaration_type() {
    let (_syntax, mut resolver) = bind_types("fn main() { let x: int = 42; }");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    assert_eq!(x_type, TypeId::INTEGER);
}
