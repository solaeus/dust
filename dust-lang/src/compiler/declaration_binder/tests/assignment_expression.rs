use crate::{
    compiler::declaration_binder::tests::{bind_declarations, find_declaration},
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn resolves_path() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x = 2; }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let reassignment = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AssignmentExpression)
        .unwrap();
    let (path, _) = reassignment.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&path.id).unwrap(), x_id);
}
