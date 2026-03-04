use crate::{resolver::type_graph::TypeNode, source::SourceFileId, syntax::SyntaxKind};

use super::bind_types;

#[test]
fn has_function_type_binding() {
    let (syntax, resolver) = bind_types("fn main() { let f = fn(x: int) -> int { x }; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let fn_exprs: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::FunctionExpression)
        .collect();
    let inner_fn = fn_exprs.last().unwrap();

    let type_id = *resolver.get_type_binding(&inner_fn.id).unwrap();
    let type_node = *resolver.types.get_type(type_id).unwrap();

    assert!(matches!(type_node, TypeNode::Function { .. }));
}
