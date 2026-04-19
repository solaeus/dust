use crate::{
    compiler::resolver::{
        declarations::Definition,
        types::{Type, TypeId},
    },
    source::{Source, SourceCode},
};

use super::{bind_declarations, find_function_body_scope};

#[test]
fn immutable() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "main() { let x = 1; }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);
    let x_symbol = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, body_scope)
        .unwrap();
    let Definition::Local {
        mutable,
        shadowed,
        type_id,
    } = x_declaration.definition
    else {
        panic!();
    };

    assert!(!mutable);
    assert!(shadowed.is_none());
    assert!(matches!(
        resolver.types.get_type(type_id).unwrap(),
        Type::Inferred { .. }
    ));
    assert_eq!(x_declaration.scope_id, body_scope);
}

#[test]
fn mutable() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn main() { let mut x = 1; }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);
    let x_symbol = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, body_scope)
        .unwrap();
    let Definition::Local {
        mutable, type_id, ..
    } = x_declaration.definition
    else {
        panic!();
    };

    assert!(mutable);
    assert!(matches!(
        resolver.types.get_type(type_id).unwrap(),
        Type::Inferred { .. }
    ));
}

#[test]
fn with_type_notation() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn main() { let x: i64 = 1; }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);
    let x_symbol = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, body_scope)
        .unwrap();
    let Definition::Local {
        mutable, type_id, ..
    } = x_declaration.definition
    else {
        panic!();
    };

    assert!(!mutable);
    assert_eq!(type_id, TypeId::I_64);
}

#[test]
fn mutable_with_type_notation() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn main() { let mut x: bool = true; }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);
    let x_symbol = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, body_scope)
        .unwrap();
    let Definition::Local {
        mutable, type_id, ..
    } = x_declaration.definition
    else {
        panic!();
    };

    assert!(mutable);
    assert_eq!(type_id, TypeId::BOOLEAN);
}

#[test]
fn shadowing() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn main() { let x = 1; let x = 2; }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);
    let x_symbol = resolver.symbols.add_symbol("x");

    let (second_x_id, second_x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, body_scope)
        .unwrap();
    let Definition::Local {
        shadowed: second_shadowed,
        ..
    } = second_x_declaration.definition
    else {
        panic!();
    };
    let first_x_id = second_shadowed.unwrap();
    let first_x_declaration = resolver.declarations.get_declaration(first_x_id).unwrap();
    let Definition::Local {
        shadowed: first_shadowed,
        ..
    } = first_x_declaration.definition
    else {
        panic!();
    };

    assert_ne!(first_x_id, second_x_id);
    assert!(first_shadowed.is_none());
}

#[test]
fn multiple() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn main() { let x = 1; let y = 2; }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);

    let x_symbol = resolver.symbols.add_symbol("x");
    let (x_id, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, body_scope)
        .unwrap();
    let y_symbol = resolver.symbols.add_symbol("y");
    let (y_id, y_declaration) = resolver
        .declarations
        .find_declaration(y_symbol, body_scope)
        .unwrap();

    assert!(matches!(x_declaration.definition, Definition::Local { .. }));
    assert!(matches!(y_declaration.definition, Definition::Local { .. }));
    assert_ne!(x_id, y_id);
}

#[test]
fn block_visibility() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn main() { { let x = 1; } }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);
    let x_symbol = resolver.symbols.add_symbol("x");
    let result = resolver.declarations.find_declaration(x_symbol, body_scope);

    assert!(
        result.is_none(),
        "let in nested block should not be visible in outer function body scope"
    );
}
