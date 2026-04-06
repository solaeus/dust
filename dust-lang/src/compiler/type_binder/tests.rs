#![allow(clippy::disallowed_methods)]

use crate::{
    compiler::tests::type_bind_function,
    resolver::{
        declarations::{Definition, Visibility},
        types::{Type, TypeId},
    },
    source::SourceFileId,
    syntax::{components::CallExpression, node::SyntaxKind},
};

#[test]
fn return_type_binding() {
    let cases = [
        "fn foo() -> bool { true }",
        "fn foo() -> char { 'a' }",
        "fn foo() -> i8 { 1 }",
        "fn foo() -> i16 { 1 }",
        "fn foo() -> i32 { 1 }",
        "fn foo() -> i64 { 1 }",
        "fn foo() -> i128 { 1 }",
        "fn foo() -> u8 { 1 }",
        "fn foo() -> u16 { 1 }",
        "fn foo() -> u32 { 1 }",
        "fn foo() -> u64 { 1 }",
        "fn foo() -> u128 { 1 }",
        "fn foo() -> f32 { 1.0 }",
        "fn foo() -> f64 { 1.0 }",
        "fn foo() -> [i32; 1] { [42] }",
        "fn foo() -> [i32; 3] { [0; 3] }",
    ];

    for source_code in cases {
        type_bind_function(source_code);
    }
}

#[test]
fn expression_forms() {
    let cases = [
        "fn foo() -> i32 { 1 + 2 }",
        "fn foo() -> bool { 1 < 2 }",
        "fn foo() -> bool { true && false }",
        "fn foo() -> bool { !true }",
        "fn foo() -> i32 { (1) }",
        "fn foo() -> i32 { if true { 1 } else { 2 } }",
        "fn foo() { while true {} }",
        "fn bar() -> i32 { 1 } fn foo() -> i32 { bar() }",
        "fn foo() -> i32 { let x: i32 = 1; x }",
        "fn foo() -> i32 { let arr:[i32; 3] = [10, 20, 30]; arr[1] }",
        "fn foo() -> i32 { 2 ^ 3 }",
        "struct Bar {} fn foo() -> Bar { Bar {} }",
        "struct Bar { x: i32 } fn foo(b: Bar) -> i32 { b.x }",
    ];

    for source_code in cases {
        type_bind_function(source_code);
    }
}

#[test]
fn empty_body() {
    type_bind_function("fn foo() {}");
}

#[test]
fn block_type_is_tail_expression() {
    type_bind_function("fn foo() -> i32 { 1.0; 2 }");
}

#[test]
fn nested_block_propagates_type() {
    type_bind_function("fn foo() -> i32 { { 1 } }");
}

#[test]
fn expression_statement_produces_unit_block() {
    type_bind_function("fn foo() { 1; }");
}

#[test]
fn generic_return_type_resolves_through_type_parameter_map() {
    let (_syntax, mut resolver, crate_scope_id) = type_bind_function("fn foo<T>() -> T { 1 }");

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let foo_declaration = *foo_declaration;

    let Definition::Function {
        type_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let type_parameter_declaration_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();
    let type_parameter_declaration_id = type_parameter_declaration_ids[0];

    let inferred_type_id = *resolver
        .type_parameter_map
        .get(&type_parameter_declaration_id)
        .unwrap();
    let resolved = resolver.resolve_type(inferred_type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}

#[test]
fn turbofish_provides_concrete_type_arguments() {
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
fn struct_expression_field_types() {
    type_bind_function("struct Foo { x: i32, y: bool } fn foo() -> Foo { Foo { x: 1, y: true } }");
}

#[test]
fn negation_preserves_type() {
    let (syntax, mut resolver, _) = type_bind_function("fn foo(x: i32) -> i32 { -x }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let neg_expr = tree
        .iter()
        .find(|n| n.node.kind == SyntaxKind::NegationExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&neg_expr.id).unwrap();
    let resolved = resolver.resolve_type(type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}

#[test]
fn generic_function_infers_type_from_argument() {
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
fn method_call_return_type() {
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

#[test]
fn range_slice_type() {
    type_bind_function("fn foo() { let arr: [i32; 5] = [1, 2, 3, 4, 5]; arr[1..3]; }");
}
