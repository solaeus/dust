use crate::{
    resolver::scopes::ScopeKind,
    source::{Source, SourceFile, SourceFileId},
    syntax::node::SyntaxKind,
};

use super::{bind_declarations, find_function_body_scope};

#[test]
fn block_creates_block_scope() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("test", "fn main() { { } }"));

    let (syntax, resolver, crate_scope_id) = bind_declarations(&source);
    let fn_body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let mut found_block_scope = false;

    for node in tree.iter() {
        if node.node.kind == SyntaxKind::BlockExpression
            && let Ok(&scope_id) = resolver.get_scope_binding(&node.id)
        {
            let scope = resolver.scopes.get_scope(scope_id).unwrap();

            if scope.kind == ScopeKind::Block && scope.parent == fn_body_scope {
                found_block_scope = true;
                break;
            }
        }
    }

    assert!(found_block_scope);
}

#[test]
fn if_branches_create_scopes() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed(
        "test",
        "fn main() { if true { } else { } }",
    ));

    let (syntax, resolver, crate_scope_id) = bind_declarations(&source);
    let fn_body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let mut block_scope_count = 0;

    for node in tree.iter() {
        if node.node.kind == SyntaxKind::BlockExpression
            && let Ok(&scope_id) = resolver.get_scope_binding(&node.id)
        {
            let scope = resolver.scopes.get_scope(scope_id).unwrap();

            if scope.kind == ScopeKind::Block && scope.parent == fn_body_scope {
                block_scope_count += 1;
            }
        }
    }

    assert_eq!(block_scope_count, 2);
}

#[test]
fn while_body_creates_scope() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed(
        "test",
        "fn main() { while true { } }",
    ));

    let (syntax, resolver, crate_scope_id) = bind_declarations(&source);
    let fn_body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let mut found_while_body_scope = false;

    for node in tree.iter() {
        if node.node.kind == SyntaxKind::BlockExpression
            && let Ok(&scope_id) = resolver.get_scope_binding(&node.id)
        {
            let scope = resolver.scopes.get_scope(scope_id).unwrap();

            if scope.kind == ScopeKind::Block && scope.parent == fn_body_scope {
                found_while_body_scope = true;
                break;
            }
        }
    }

    assert!(found_while_body_scope);
}

#[test]
fn path_expression_binds_declaration() {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed(
        "test",
        "fn main() { let x = 1; x; }",
    ));

    let (syntax, resolver, _crate_scope_id) = bind_declarations(&source);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let path_expr = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::PathExpression)
        .unwrap();

    let declaration_id = resolver.get_declaration_binding(&path_expr.id).unwrap();
    let declaration = resolver
        .declarations
        .get_declaration(*declaration_id)
        .unwrap();

    assert!(matches!(
        declaration.definition,
        crate::resolver::declarations::Definition::Local { .. }
    ));
}
