use crate::{
    compiler::declaration_binder::tests::{bind_declarations, find_declaration},
    resolver::declaration_graph::DeclarationKind,
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn resolves_to_local_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x }");

    let (x_id, x_kind) = find_declaration(&mut resolver, "x").unwrap();

    assert!(matches!(x_kind, DeclarationKind::Local { .. }));

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let path_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::PathExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&path_expr.id).unwrap(),
        x_id
    );
}

#[test]
fn qualified_path_resolves_through_module() {
    let (_syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } fn main() { foo::bar }");

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .unwrap();

    assert!(matches!(
        bar_declaration.kind,
        DeclarationKind::Function { .. }
    ));
}

#[test]
fn resolves_from_outer_scope() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; { x } }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let path_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::PathExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&path_expr.id).unwrap(),
        x_id
    );
}

#[test]
fn resolves_through_nested_modules() {
    let (syntax, mut resolver) =
        bind_declarations("mod a { pub mod b { pub fn c() {} } } fn main() { a::b::c }");

    let (c_id, _) = find_declaration(&mut resolver, "c").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let path_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::PathExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&path_expr.id).unwrap(),
        c_id
    );
}
