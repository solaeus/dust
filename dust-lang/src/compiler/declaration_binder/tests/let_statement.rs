use crate::{
    resolver::declaration_graph::{DeclarationId, DeclarationKind},
    source::SourceFileId,
    syntax::SyntaxKind,
};

use super::{bind_declarations, find_declaration};

#[test]
fn creates_local_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 42; }");

    let (x_id, x_kind) = find_declaration(&mut resolver, "x").unwrap();

    assert!(matches!(x_kind, DeclarationKind::Local { .. }));

    let x_declaration = resolver.declarations.get_declaration(x_id).unwrap();

    assert!(!x_declaration.public);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let let_stmt = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::LetStatement)
        .unwrap();
    let (path, _) = let_stmt.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&path.id).unwrap(), x_id);
}

#[test]
fn same_scope_shadowing_creates_distinct_declarations() {
    let (_syntax, mut resolver) = bind_declarations("fn main() { let x = 1; let x = 2; }");

    let x_symbol_id = resolver.symbols.add_symbol("x");
    let x_declarations: Vec<DeclarationId> = resolver
        .declarations
        .iter()
        .filter(|(_, declaration)| declaration.symbol_id == x_symbol_id)
        .map(|(id, _)| id)
        .collect();

    assert_eq!(
        x_declarations.len(),
        2,
        "Expected two distinct declarations for shadowed local 'x'"
    );
    assert_ne!(
        x_declarations[0], x_declarations[1],
        "Shadowed declarations must have different DeclarationIds"
    );
}

#[test]
fn shadowed_rhs_resolves_to_original() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; let x = x + 1; }");

    let x_symbol_id = resolver.symbols.add_symbol("x");
    let x_declarations: Vec<DeclarationId> = resolver
        .declarations
        .iter()
        .filter(|(_, declaration)| declaration.symbol_id == x_symbol_id)
        .map(|(id, _)| id)
        .collect();
    let first_x = x_declarations[0];

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let path_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::PathExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&path_expr.id).unwrap(),
        first_x,
        "RHS 'x' in shadowing let must resolve to the original declaration"
    );
}
