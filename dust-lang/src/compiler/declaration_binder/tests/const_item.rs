use crate::{
    compiler::resolver::{declarations::Definition, types::TypeId},
    source::{Source, SourceCode},
};

use super::bind_declarations;

#[test]
fn simple() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed("test", "const X: i64 = 42;"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let x_symbol = resolver.symbols.add_symbol("X");
    let (_, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, crate_scope_id)
        .unwrap();
    let Definition::Constant { public, type_id } = x_declaration.definition else {
        panic!();
    };

    assert!(!public);
    assert_eq!(type_id, TypeId::I_64);
    assert_eq!(x_declaration.scope_id, crate_scope_id);
}

#[test]
fn value_expression_scoped() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "const X: i64 = { let y = 1; y };",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let x_symbol = resolver.symbols.add_symbol("X");
    let (_, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, crate_scope_id)
        .unwrap();
    let Definition::Constant { public, type_id } = x_declaration.definition else {
        panic!();
    };

    assert!(!public);
    assert_eq!(type_id, TypeId::I_64);

    let y_symbol = resolver.symbols.add_symbol("y");
    let result = resolver
        .declarations
        .find_declaration(y_symbol, crate_scope_id);

    assert!(result.is_none());
}
