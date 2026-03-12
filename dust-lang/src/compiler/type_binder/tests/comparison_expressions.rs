use crate::{
    compiler::type_binder::tests::bind_types, resolver::type_graph::TypeId, source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn equal_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { 1 == 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::EqualExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn not_equal_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { 1 != 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::NotEqualExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn less_than_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { 1 < 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::LessThanExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn greater_than_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { 1 > 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::GreaterThanExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}
