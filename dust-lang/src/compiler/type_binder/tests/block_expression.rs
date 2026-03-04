use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn empty_has_unit_type() {
    let (syntax, resolver) = bind_types("fn main() { {} }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::BlockExpression)
        .collect();
    let inner_block = blocks.last().unwrap();

    assert_eq!(
        *resolver.get_type_binding(&inner_block.id).unwrap(),
        TypeId::UNIT
    );
}

#[test]
fn with_expression_has_expression_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { { 42 } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::BlockExpression)
        .collect();
    let inner_block = blocks.last().unwrap();

    assert_eq!(
        *resolver.get_type_binding(&inner_block.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn with_statement_has_unit_type() {
    let (syntax, resolver) = bind_types("fn main() { { let x = 42; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::BlockExpression)
        .collect();
    let inner_block = blocks.last().unwrap();

    assert_eq!(
        *resolver.get_type_binding(&inner_block.id).unwrap(),
        TypeId::UNIT
    );
}
