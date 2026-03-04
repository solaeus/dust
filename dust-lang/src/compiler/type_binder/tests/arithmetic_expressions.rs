use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn addition_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { 1 + 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AdditionExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn subtraction_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { 3 - 1 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::SubtractionExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn multiplication_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { 2 * 3 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::MultiplicationExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn division_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { 6 / 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::DivisionExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}
