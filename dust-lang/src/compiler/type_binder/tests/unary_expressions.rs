use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn negation_has_integer_type() {
    let (syntax, resolver) = bind_types("fn main() -> i64 { -42 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::NegationExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::I_64
    );
}

#[test]
fn logical_not_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { !true }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::NotExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn float_negation_has_float_type() {
    let (syntax, resolver) = bind_types("fn main() -> float { -42.0 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::NegationExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::F_64);
}
