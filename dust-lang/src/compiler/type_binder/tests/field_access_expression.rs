use crate::{
    compiler::{
        resolver::{declarations::Definition, types::TypeId},
        tests::type_bind_function,
    },
    source::SourceCodeId,
    syntax::{
        components::{FieldAccessExpression, SyntaxComponent},
        node::SyntaxKind,
    },
};

#[test]
fn binds_field_declaration() {
    let (syntax, mut resolver, crate_scope_id) =
        type_bind_function("struct Foo { x: i64 } fn foo(f: Foo) -> i64 { f.x }");

    let tree = syntax.get_tree(SourceCodeId::MAIN).unwrap();
    let field_access = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::FieldAccessExpression)
        .unwrap();
    let field_access_component = FieldAccessExpression::from_reader(&field_access).unwrap();

    let field_declaration_id = *resolver
        .get_declaration_binding(&field_access_component.field_name.id)
        .unwrap();
    let field_declaration = resolver.declarations.get_declaration(field_declaration_id);

    let foo_symbol = resolver.symbols.add_symbol("Foo");
    let foo_declaration_id = *resolver
        .declarations
        .find_declaration_id(foo_symbol, crate_scope_id)
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
    assert_eq!(parent_struct, foo_declaration_id);
    assert_eq!(type_id, TypeId::I_64);
}

#[test]
fn struct_field() {
    type_bind_function("struct Bar { x: i32 } fn foo(b: Bar) -> i32 { b.x }");
}
