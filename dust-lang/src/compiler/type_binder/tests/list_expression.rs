use crate::{
    resolver::type_graph::{TypeId, TypeNode},
    source::SourceFileId,
    syntax::SyntaxKind,
};

use super::bind_types;

#[test]
fn creates_list_type() {
    let (syntax, resolver) = bind_types("fn main() -> [int] { [1, 2, 3] }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ListExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&node.id).unwrap();
    let type_node = *resolver.types.get_type(type_id).unwrap();

    let element_type = match type_node {
        TypeNode::List { element_type } => element_type,
        other => panic!("expected List type, got {other:?}"),
    };

    assert_eq!(element_type, TypeId::I_64);
}

#[test]
fn empty_list_creates_inferred_type() {
    let (syntax, resolver) = bind_types("fn main() { let x: [int] = []; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ListExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&node.id).unwrap();
    let type_node = *resolver.types.get_type(type_id).unwrap();

    assert!(matches!(type_node, TypeNode::List { .. }));
}

#[test]
fn index_has_element_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { [1, 2, 3][0] }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::IndexExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::I_64
    );
}
