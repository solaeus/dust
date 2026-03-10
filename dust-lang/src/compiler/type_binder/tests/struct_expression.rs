use crate::{
    compiler::type_binder::tests::bind_types,
    resolver::type_graph::TypeNode,
    source::SourceFileId,
    syntax::node::SyntaxKind,
};

#[test]
fn has_struct_type() {
    let (syntax, resolver) = bind_types("struct Foo { x: i64 } fn main() { Foo { x: 1 } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&node.id).unwrap();
    let type_node = *resolver.types.get_type(type_id).unwrap();

    assert!(matches!(type_node, TypeNode::Struct { .. }));
}
