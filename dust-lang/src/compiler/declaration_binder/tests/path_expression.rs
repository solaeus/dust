use crate::{
    resolver::declaration_graph::DeclarationKind, source::SourceFileId, syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn resolves_to_local_declaration() {
    let (_syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x }");

    let (x_declaration_id, _) = find_declaration(&mut resolver, "x").unwrap();

    assert!(matches!(
        resolver
            .declarations
            .get_declaration(x_declaration_id)
            .unwrap()
            .kind,
        DeclarationKind::Local
    ));
}

#[test]
fn binds_to_referenced_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x }");
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
fn qualified_path_resolves_through_module() {
    let (_syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } fn main() { foo::bar }");

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .unwrap();

    assert_eq!(bar_declaration.kind, DeclarationKind::Function);
}
