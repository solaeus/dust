use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn has_declaration_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { let x = 42; x }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::PathExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::I_64
    );
}
