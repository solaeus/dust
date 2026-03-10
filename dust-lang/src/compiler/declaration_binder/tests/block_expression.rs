use crate::{
    compiler::declaration_binder::tests::bind_declarations,
    resolver::scope_graph::ScopeKind,
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn nested_blocks_create_scope_chain() {
    let (_syntax, mut resolver) =
        bind_declarations("fn main() { let a = 1; { let b = 2; { let c = 3; } } }");

    let a_symbol_id = resolver.symbols.add_symbol("a");
    let b_symbol_id = resolver.symbols.add_symbol("b");
    let c_symbol_id = resolver.symbols.add_symbol("c");

    let a_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == a_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let b_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == b_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let c_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == c_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let b_scope = resolver.scopes.get_scope(b_scope_id).unwrap();
    let c_scope = resolver.scopes.get_scope(c_scope_id).unwrap();

    assert_eq!(b_scope.parent, a_scope_id);
    assert_eq!(c_scope.parent, b_scope_id);
}

#[test]
fn binds_to_block_scope() {
    let (syntax, mut resolver) = bind_declarations("fn main() { { let x = 1; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks = tree
        .iter()
        .filter(|reader| reader.kind() == SyntaxKind::BlockExpression)
        .collect::<Vec<_>>();
    let inner_block = blocks.last().unwrap();

    let scope_id = resolver.get_scope_binding(&inner_block.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Block);

    let x_symbol_id = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == x_symbol_id)
        .unwrap();
    let x_scope = resolver.scopes.get_scope(x_declaration.scope_id).unwrap();

    assert_eq!(x_scope.kind, ScopeKind::Block);
}
