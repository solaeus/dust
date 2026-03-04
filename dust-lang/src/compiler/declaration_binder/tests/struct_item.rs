use crate::{
    resolver::declaration_graph::DeclarationKind, source::SourceFileId, syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_type_declaration() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int, y: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    assert!(matches!(foo_kind, DeclarationKind::Type { .. }));
}

#[test]
fn field_count_matches_definition() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int, y: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    let members = match foo_kind {
        DeclarationKind::Type { members, .. } => members,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(members.count, 2);
}

#[test]
fn fields_reference_parent() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int }");

    let (foo_declaration_id, _) = find_declaration(&mut resolver, "Foo").unwrap();

    let (_, x_kind) = find_declaration(&mut resolver, "x").unwrap();

    let parent_id = match x_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration for field, got {other:?}"),
    };

    assert_eq!(parent_id, Some(foo_declaration_id));
}

#[test]
fn has_no_parent() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    let parent_id = match foo_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent_id, None);
}

#[test]
fn name_binds_to_type_declaration() {
    let (syntax, mut resolver) = bind_declarations("struct Foo { x: int }");
    let (foo_id, _) = find_declaration(&mut resolver, "Foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructItem)
        .unwrap();
    let (name, _) = item.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), foo_id);
}

#[test]
fn field_name_binds_to_field_declaration() {
    let (syntax, mut resolver) = bind_declarations("struct Foo { x: int }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let fields = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructFieldsDeclaration)
        .unwrap();
    let name = fields.children().unwrap().next().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), x_id);
}

#[test]
fn is_not_public_by_default() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int }");

    let foo_symbol_id = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(!foo_declaration.is_public);
}

#[test]
fn is_public_when_pub() {
    let (_syntax, mut resolver) = bind_declarations("pub struct Foo { x: int }");

    let foo_symbol_id = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(foo_declaration.is_public);
}
