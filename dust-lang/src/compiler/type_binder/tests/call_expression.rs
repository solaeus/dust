use crate::{
    compiler::type_binder::tests::bind_types, resolver::type_graph::TypeId, source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn has_return_type() {
    let (syntax, resolver) = bind_types("fn foo() -> i64 { 42 } fn main() -> i64 { foo() }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::CallExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::I_64);
}
