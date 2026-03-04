use crate::{
    resolver::declaration_graph::DeclarationKind, source::SourceFileId, syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_type_declaration() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red, Green, Blue }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    assert!(matches!(color_kind, DeclarationKind::Type { .. }));
}

#[test]
fn variant_count_matches_definition() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red, Green, Blue }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    let members = match color_kind {
        DeclarationKind::Type { members, .. } => members,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(members.count, 3);
}

#[test]
fn variants_reference_parent() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red }");

    let (color_declaration_id, _) = find_declaration(&mut resolver, "Color").unwrap();

    let (_, red_kind) = find_declaration(&mut resolver, "Red").unwrap();

    let parent_id = match red_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration for variant, got {other:?}"),
    };

    assert_eq!(parent_id, Some(color_declaration_id));
}

#[test]
fn has_no_parent() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    let parent_id = match color_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent_id, None);
}

#[test]
fn name_binds_to_type_declaration() {
    let (syntax, mut resolver) = bind_declarations("enum Color { Red }");
    let (color_id, _) = find_declaration(&mut resolver, "Color").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::EnumItem)
        .unwrap();
    let mut children = item.children().unwrap();
    let name = children.next().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&name.id).unwrap(),
        color_id
    );
}

#[test]
fn variant_name_binds_to_variant_declaration() {
    let (syntax, mut resolver) = bind_declarations("enum Color { Red }");
    let (red_id, _) = find_declaration(&mut resolver, "Red").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let variant = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::EnumVariant)
        .unwrap();
    let name = variant.child().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), red_id);
}

#[test]
fn is_not_public_by_default() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red }");

    let color_symbol_id = resolver.symbols.add_symbol("Color");
    let (_, color_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == color_symbol_id)
        .unwrap();

    assert!(!color_declaration.is_public);
}
