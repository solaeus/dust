#![allow(clippy::disallowed_methods)]

use crate::{
    compiler::type_binder::TypeBinder,
    error::ErrorKind,
    resolver::{
        Resolver,
        declarations::{Definition, Visibility},
        scopes::ScopeId,
        types::{Type, TypeId},
    },
    source::{Source, SourceFile},
    syntax::{Syntax, components::FunctionItem},
};

fn type_bind_function(source_code: &str) -> (Syntax, Resolver, ScopeId) {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", source_code));

    let (syntax, mut resolver, crate_scope_id) = crate::compiler::tests::bind_declarations(&source);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let foo_declaration = *foo_declaration;

    let Definition::Function {
        type_parameters,
        return_type_id,
        ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let (position, syntax_id) = foo_declaration.syntax.unwrap();

    let function_item = syntax
        .get_tree(position.file_id)
        .and_then(|tree| tree.get_node(syntax_id))
        .unwrap();
    let FunctionItem { body, .. } = function_item.as_component().unwrap();

    resolver.type_parameter_map.clear();

    let type_parameter_declaration_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();

    for &type_parameter_declaration_id in type_parameter_declaration_ids {
        let inferred_type_id = resolver.types.create_inferred_type(None);
        resolver
            .type_parameter_map
            .insert(type_parameter_declaration_id, inferred_type_id);
    }

    let mut errors = Vec::new();
    let mut type_binder = TypeBinder::new(&syntax, &mut resolver, &mut errors);

    match type_binder.bind_function_body(body, return_type_id) {
        Ok(()) => {}
        Err(error) => errors.push(ErrorKind::Compile(error)),
    }

    assert!(errors.is_empty(), "{source_code}: {errors:#?}");

    (syntax, resolver, crate_scope_id)
}

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
    let resolved = resolver.resolve_type_through_map(inferred_type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}
