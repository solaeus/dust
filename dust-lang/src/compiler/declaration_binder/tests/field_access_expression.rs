use crate::{
    resolver::{
        declarations::{Definition, Visibility},
        types::TypeId,
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{
        components::{FieldAccessExpression, SyntaxComponent},
        node::SyntaxKind,
    },
};

use super::bind_declarations;

#[test]
fn field_access_binds_field_declaration() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed(
        "test",
        "struct Foo { x: i64 } fn bar() { let f: Foo = Foo { x: 1 }; f.x; }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let field_access = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::FieldAccessExpression)
        .unwrap();

    let field_access_component = FieldAccessExpression::from_reader(&field_access).unwrap();

    let field_declaration_id = *resolver
        .get_declaration_binding(&field_access_component.field_name.id)
        .unwrap();
    let field_declaration = resolver
        .declarations
        .get_declaration(field_declaration_id)
        .unwrap();

    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let (foo_id, _) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    let Definition::Field {
        parent_struct,
        type_id,
        ..
    } = field_declaration.definition
    else {
        panic!();
    };

    let x_symbol = resolver.symbols.add_symbol("x");

    assert_eq!(field_declaration.symbol_id, x_symbol);
    assert_eq!(parent_struct, foo_id);
    assert_eq!(type_id, TypeId::I_64);
}
