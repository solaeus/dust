use crate::{
    resolver::declaration_graph::DeclarationKind, source::SourceFileId, syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_type_declaration() {
    let (syntax, mut resolver) = bind_declarations("struct Foo { x: i64, y: i64 }");

    let (foo_id, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    let (parent, members) = match foo_kind {
        DeclarationKind::Type {
            parent, members, ..
        } => (parent, members),
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent, None);
    assert_eq!(members.count, 2);

    let foo_declaration = resolver.declarations.get_declaration(foo_id).unwrap();

    assert!(!foo_declaration.public);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructItem)
        .unwrap();
    let (name, _) = item.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), foo_id);

    let (x_id, x_kind) = find_declaration(&mut resolver, "x").unwrap();
    let x_parent = match x_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration for field, got {other:?}"),
    };

    assert_eq!(x_parent, Some(foo_id));

    let fields = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructFieldsDeclaration)
        .unwrap();
    let field_name = fields.children().unwrap().next().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&field_name.id).unwrap(),
        x_id
    );
}

#[test]
fn is_public_when_pub() {
    let (_syntax, mut resolver) = bind_declarations("pub struct Foo { x: i64 }");

    let foo_symbol_id = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(foo_declaration.public);
}
