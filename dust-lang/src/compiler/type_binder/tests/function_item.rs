use crate::{
    compiler::resolver::{declarations::Definition, types::TypeId},
    compiler::tests::type_bind_function,
};

#[test]
fn empty_body() {
    type_bind_function("fn foo() {}");
}

#[test]
fn return_bool() {
    type_bind_function("fn foo() -> bool { true }");
}

#[test]
fn return_char() {
    type_bind_function("fn foo() -> char { 'a' }");
}

#[test]
fn return_i8() {
    type_bind_function("fn foo() -> i8 { 1 }");
}

#[test]
fn return_i16() {
    type_bind_function("fn foo() -> i16 { 1 }");
}

#[test]
fn return_i32() {
    type_bind_function("fn foo() -> i32 { 1 }");
}

#[test]
fn return_i64() {
    type_bind_function("fn foo() -> i64 { 1 }");
}

#[test]
fn return_i128() {
    type_bind_function("fn foo() -> i128 { 1 }");
}

#[test]
fn return_u8() {
    type_bind_function("fn foo() -> u8 { 1 }");
}

#[test]
fn return_u16() {
    type_bind_function("fn foo() -> u16 { 1 }");
}

#[test]
fn return_u32() {
    type_bind_function("fn foo() -> u32 { 1 }");
}

#[test]
fn return_u64() {
    type_bind_function("fn foo() -> u64 { 1 }");
}

#[test]
fn return_u128() {
    type_bind_function("fn foo() -> u128 { 1 }");
}

#[test]
fn return_f32() {
    type_bind_function("fn foo() -> f32 { 1.0 }");
}

#[test]
fn return_f64() {
    type_bind_function("fn foo() -> f64 { 1.0 }");
}

#[test]
fn return_array() {
    type_bind_function("fn foo() -> [i32; 1] { [42] }");
}

#[test]
fn return_array_repeat() {
    type_bind_function("fn foo() -> [i32; 3] { [0; 3] }");
}

#[test]
fn generic_return_type_resolves_through_type_parameter_map() {
    let (_syntax, mut resolver, crate_scope_id) = type_bind_function("fn foo<T>() -> T { 1 }");

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let foo_declaration_id = *resolver
        .declarations
        .find_declaration_id(foo_symbol, crate_scope_id)
        .unwrap();
    let foo_declaration = resolver
        .declarations
        .get_declaration(foo_declaration_id)
        .unwrap();

    let Definition::Function {
        type_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let type_parameter_entries = resolver
        .scopes
        .get_members(type_parameters.unwrap());
    let type_parameter_declaration_id = type_parameter_entries[0].1;

    let inferred_type_id = *resolver
        .type_parameter_map
        .get(&type_parameter_declaration_id)
        .unwrap();
    let resolved = resolver.resolve_type(inferred_type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}
