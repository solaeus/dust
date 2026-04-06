use crate::{
    compiler::tests::type_bind_function,
    resolver::types::{Type, TypeId},
    source::SourceFileId,
    syntax::{components::CallExpression, node::SyntaxKind},
};

#[test]
fn simple() {
    type_bind_function("fn bar() -> i32 { 1 } fn foo() -> i32 { bar() }");
}

#[test]
fn turbofish_type_arguments() {
    let (syntax, resolver, _) =
        type_bind_function("fn bar<T>(x: T) -> T { x } fn foo() -> i32 { bar::<i32>(1) }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call_expression = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::CallExpression)
        .unwrap();

    let CallExpression { callee, .. } = call_expression.as_component().unwrap();
    let type_id = *resolver.get_type_binding(&callee.id).unwrap();
    let r#type = *resolver.types.get_type(type_id).unwrap();

    let Type::FunctionDefinition { type_arguments, .. } = r#type else {
        panic!("expected FunctionDefinition, got {type:?}");
    };

    let type_argument_ids = resolver.types.get_type_members(type_arguments).unwrap();

    assert_eq!(type_argument_ids.len(), 1);
    assert_eq!(type_argument_ids[0], TypeId::I_32);
}

#[test]
fn generic_infers_type_from_argument() {
    let (syntax, mut resolver, _) =
        type_bind_function("fn bar<T>(x: T) -> T { x } fn foo() -> i32 { bar(1) }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call_expr = tree
        .iter()
        .find(|n| n.node.kind == SyntaxKind::CallExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&call_expr.id).unwrap();
    let resolved = resolver.resolve_type(type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}

#[test]
fn method_return_type() {
    let (syntax, mut resolver, _) = type_bind_function(
        "struct Foo {} impl Foo { fn bar(self) -> i32 { 1 } } fn foo(f: Foo) -> i32 { f.bar() }",
    );

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call_expr = tree
        .iter()
        .find(|n| n.node.kind == SyntaxKind::CallExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&call_expr.id).unwrap();
    let resolved = resolver.resolve_type(type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}
