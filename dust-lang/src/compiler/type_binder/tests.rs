#![allow(clippy::disallowed_methods)]

use crate::{
    compiler::tests::type_bind_function,
    resolver::{
        declarations::{Definition, Visibility},
        types::TypeId,
    },
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
    type_bind_function("fn foo() -> i32 { 1; 2 }");
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
