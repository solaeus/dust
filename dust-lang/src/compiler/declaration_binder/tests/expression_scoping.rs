use crate::{
    compiler::resolver::{declarations::Definition, scopes::ScopeKind},
    source::{Source, SourceCode, SourceFileId},
    syntax::{
        components::{StructExpression, StructExpressionStructFields, SyntaxComponent},
        node::SyntaxKind,
    },
};

use super::{bind_declarations, find_function_body_scope};

#[test]
fn block_creates_block_scope() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "fn main() { { } }"));

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

    source.add_file(SourceCode::validated_borrowed(
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

    source.add_file(SourceCode::validated_borrowed(
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

    source.add_file(SourceCode::validated_borrowed(
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
        crate::compiler::resolver::declarations::Definition::Local { .. }
    ));
}

#[test]
fn struct_expression_binds_field_name() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Foo { x: i64 } fn main() { Foo { x: 1 }; }",
    ));

    let (syntax, resolver, _crate_scope_id) = bind_declarations(&source);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let struct_expr = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::StructExpression)
        .unwrap();

    let StructExpression { fields, .. } = StructExpression::from_reader(&struct_expr).unwrap();
    let StructExpressionStructFields {
        name_expression_pairs,
    } = StructExpressionStructFields::from_reader(&fields).unwrap();

    for [field_name, _] in name_expression_pairs {
        let declaration_id = resolver.get_declaration_binding(&field_name.id).unwrap();
        let declaration = resolver
            .declarations
            .get_declaration(*declaration_id)
            .unwrap();

        assert!(matches!(declaration.definition, Definition::Field { .. }));
    }
}

#[test]
fn struct_expression_binds_multiple_field_names() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Foo { x: i64, y: i64 } fn main() { Foo { x: 1, y: 2 }; }",
    ));

    let (syntax, mut resolver, _crate_scope_id) = bind_declarations(&source);

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let struct_expr = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::StructExpression)
        .unwrap();

    let StructExpression { fields, .. } = StructExpression::from_reader(&struct_expr).unwrap();
    let StructExpressionStructFields {
        name_expression_pairs,
    } = StructExpressionStructFields::from_reader(&fields).unwrap();

    let x_symbol = resolver.symbols.add_symbol("x");
    let y_symbol = resolver.symbols.add_symbol("y");
    let mut bound_symbols = Vec::new();

    for [field_name, _] in name_expression_pairs {
        let declaration_id = resolver.get_declaration_binding(&field_name.id).unwrap();
        let declaration = resolver
            .declarations
            .get_declaration(*declaration_id)
            .unwrap();

        assert!(matches!(declaration.definition, Definition::Field { .. }));
        bound_symbols.push(declaration.symbol_id);
    }

    assert_eq!(bound_symbols.len(), 2);
    assert_eq!(bound_symbols[0], x_symbol);
    assert_eq!(bound_symbols[1], y_symbol);
}
