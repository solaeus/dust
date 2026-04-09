use crate::{
    compiler::{resolver::types::TypeId, tests::type_bind_function},
    source::FileId,
    syntax::node::SyntaxKind,
};

#[test]
fn preserves_type() {
    let (syntax, mut resolver, _) = type_bind_function("fn foo(x: i32) -> i32 { -x }");

    let tree = syntax.get_tree(FileId::MAIN).unwrap();
    let neg_expr = tree
        .iter()
        .find(|n| n.node.kind == SyntaxKind::NegationExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&neg_expr.id).unwrap();
    let resolved = resolver.resolve_type(type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}
