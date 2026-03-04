use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn addition_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> i64 { 1 + 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AdditionExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::I_64);
}

#[test]
fn subtraction_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> i64 { 3 - 1 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::SubtractionExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::I_64);
}

#[test]
fn multiplication_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> i64 { 2 * 3 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::MultiplicationExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::I_64);
}

#[test]
fn division_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> i64 { 6 / 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::DivisionExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::I_64);
}

#[test]
fn float_addition_has_float_type() {
    let (syntax, resolver) = bind_types("fn main() -> float { 1.0 + 2.0 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AdditionExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::F_64);
}

#[test]
fn modulo_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> i64 { 6 % 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ModuloExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::I_64);
}

#[test]
fn string_character_concatenation_returns_string() {
    let (syntax, resolver) = bind_types("fn main() -> str { \"hello\" + 'w' }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AdditionExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::STRING
    );
}
