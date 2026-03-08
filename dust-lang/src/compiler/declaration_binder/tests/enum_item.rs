use crate::{
    resolver::declaration_graph::DeclarationKind, source::SourceFileId, syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_type_declaration() {
    let (syntax, mut resolver) = bind_declarations("enum Color { Red, Green, Blue }");

    let (color_id, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    let (parent, members) = match color_kind {
        DeclarationKind::Type {
            parent, members, ..
        } => (parent, members),
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent, None);
    assert_eq!(members.count, 3);

    let color_declaration = resolver.declarations.get_declaration(color_id).unwrap();

    assert!(!color_declaration.public);

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

    let (_, red_kind) = find_declaration(&mut resolver, "Red").unwrap();
    let red_parent = match red_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration for variant, got {other:?}"),
    };

    assert_eq!(red_parent, Some(color_id));

    let variant = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::EnumVariant)
        .unwrap();
    let variant_name = variant.child().unwrap();
    let (red_id, _) = find_declaration(&mut resolver, "Red").unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&variant_name.id).unwrap(),
        red_id
    );
}

#[test]
fn is_public_when_pub() {
    let (_syntax, mut resolver) = bind_declarations("pub enum Color { Red }");

    let color_symbol_id = resolver.symbols.add_symbol("Color");
    let (_, color_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == color_symbol_id)
        .unwrap();

    assert!(color_declaration.public);
}
