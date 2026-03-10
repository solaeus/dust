use crate::{
    compiler::type_binder::tests::bind_types,
    resolver::type_graph::TypeId,
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn and_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { true && false }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AndExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn or_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { true || false }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::OrExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}
