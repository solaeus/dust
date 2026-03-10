use crate::{
    compiler::declaration_binder::tests::{bind_declarations, find_declaration},
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn resolves_callee() {
    let (syntax, mut resolver) = bind_declarations("fn foo() {} fn main() { foo() }");
    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::CallExpression)
        .unwrap();
    let (callee, _) = call.binary_children().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&callee.id).unwrap(),
        foo_id
    );
}
