use crate::{resolver::type_graph::TypeId, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn has_unit_type() {
    let (syntax, resolver) = bind_types("fn main() { let x = 1; x = 2; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ReassignmentStatement)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::UNIT);
}
