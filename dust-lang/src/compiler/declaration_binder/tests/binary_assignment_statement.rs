use crate::source::SourceFileId;
use crate::syntax::SyntaxKind;

use super::{bind_declarations, find_declaration};

#[test]
fn resolves_path() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x += 2; }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let assignment = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AdditionAssignmentStatement)
        .unwrap();
    let (path, _) = assignment.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&path.id).unwrap(), x_id);
}
