use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn integer_has_integer_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { 42 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::IntegerExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::I_64);
}

#[test]
fn float_has_float_type() {
    let (syntax, resolver) = bind_types("fn main() -> float { 42.0 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::FloatExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::F_64);
}

#[test]
fn boolean_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { true }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::BooleanExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn string_has_string_type() {
    let (syntax, resolver) = bind_types("fn main() -> str { \"hello\" }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StringExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::STRING
    );
}

#[test]
fn character_has_character_type() {
    let (syntax, resolver) = bind_types("fn main() -> char { 'a' }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::CharacterExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::CHARACTER
    );
}
