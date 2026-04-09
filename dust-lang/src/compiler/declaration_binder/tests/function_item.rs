use crate::{
    compiler::resolver::{
        declarations::{Definition, Visibility},
        scopes::ScopeKind,
        types::TypeId,
    },
    source::{Code, Source},
};

use super::{bind_declarations, find_function_body_scope};

#[test]
fn empty() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed("test", "fn foo() {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        public,
        type_parameters,
        value_parameters,
        return_type_id,
    } = foo_declaration.definition
    else {
        panic!();
    };

    assert!(!public);
    assert!(type_parameters.is_empty());
    assert!(value_parameters.is_empty());
    assert_eq!(return_type_id, TypeId::UNIT);
    assert_eq!(foo_declaration.scope_id, crate_scope_id);
}

#[test]
fn with_generics_parameters_and_return_type() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "pub fn foo<A, B>(x: i64, y: bool) -> i64 {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        public,
        type_parameters,
        value_parameters,
        return_type_id,
    } = foo_declaration.definition
    else {
        panic!();
    };
    let type_parameter_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();

    assert!(public);
    assert_eq!(return_type_id, TypeId::I_64);
    assert_eq!(type_parameters.len(), 2);

    for &id in type_parameter_ids {
        let declaration = resolver.declarations.get_declaration(id).unwrap();

        assert!(matches!(declaration.definition, Definition::TypeParameter));
    }

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");
    let first = resolver
        .declarations
        .get_declaration(type_parameter_ids[0])
        .unwrap();
    let second = resolver
        .declarations
        .get_declaration(type_parameter_ids[1])
        .unwrap();

    assert_eq!(first.symbol_id, a_symbol);
    assert_eq!(second.symbol_id, b_symbol);

    let value_parameter_types = resolver.types.get_type_members(value_parameters).unwrap();

    assert_eq!(value_parameter_types, &[TypeId::I_64, TypeId::BOOLEAN]);
}

#[test]
fn parameters_not_visible_in_declaring_scope() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed("test", "fn foo(x: i64) {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let x_symbol = resolver.symbols.add_symbol("x");
    let result =
        resolver
            .declarations
            .find_declaration(x_symbol, crate_scope_id, Visibility::Block);

    assert!(
        result.is_none(),
        "parameter should not be visible in the declaring scope"
    );
}

#[test]
fn same_name_in_different_modules() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "mod a { fn foo() -> i64 {} } mod b { fn foo() -> bool {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    let a_symbol = resolver.symbols.add_symbol("a");
    let (_, a_declaration) = resolver
        .declarations
        .find_declaration(a_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: a_scope,
        ..
    } = a_declaration.definition
    else {
        panic!();
    };

    let b_symbol = resolver.symbols.add_symbol("b");
    let (_, b_declaration) = resolver
        .declarations
        .find_declaration(b_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: b_scope,
        ..
    } = b_declaration.definition
    else {
        panic!();
    };

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (a_foo_id, a_foo) = resolver
        .declarations
        .find_declaration(foo_symbol, a_scope, Visibility::Module)
        .unwrap();
    let Definition::Function {
        return_type_id: a_return,
        ..
    } = a_foo.definition
    else {
        panic!();
    };

    assert_eq!(a_return, TypeId::I_64);

    let (b_foo_id, b_foo) = resolver
        .declarations
        .find_declaration(foo_symbol, b_scope, Visibility::Module)
        .unwrap();
    let Definition::Function {
        return_type_id: b_return,
        ..
    } = b_foo.definition
    else {
        panic!();
    };

    assert_eq!(b_return, TypeId::BOOLEAN);
    assert_ne!(a_foo_id, b_foo_id);
}

#[test]
fn type_parameters_have_correct_identity() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed("test", "fn foo<A, B>() {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        type_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let type_parameter_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();

    assert_eq!(type_parameters.len(), 2);

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");

    let first = resolver
        .declarations
        .get_declaration(type_parameter_ids[0])
        .unwrap();
    let second = resolver
        .declarations
        .get_declaration(type_parameter_ids[1])
        .unwrap();

    assert_eq!(first.symbol_id, a_symbol);
    assert!(matches!(first.definition, Definition::TypeParameter));
    assert_eq!(second.symbol_id, b_symbol);
    assert!(matches!(second.definition, Definition::TypeParameter));
}

#[test]
fn value_parameter_declarations() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "fn foo(x: i64, y: bool) {}",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let fn_body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);

    let x_symbol = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .find_declaration(x_symbol, fn_body_scope, Visibility::Block)
        .unwrap();
    let Definition::Local {
        mutable: x_mutable,
        type_id: x_type,
        ..
    } = x_declaration.definition
    else {
        panic!();
    };

    assert!(!x_mutable);
    assert_eq!(x_type, TypeId::I_64);

    let y_symbol = resolver.symbols.add_symbol("y");
    let (_, y_declaration) = resolver
        .declarations
        .find_declaration(y_symbol, fn_body_scope, Visibility::Block)
        .unwrap();
    let Definition::Local {
        mutable: y_mutable,
        type_id: y_type,
        ..
    } = y_declaration.definition
    else {
        panic!();
    };

    assert!(!y_mutable);
    assert_eq!(y_type, TypeId::BOOLEAN);
}

#[test]
fn function_body_creates_function_scope() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed("test", "fn foo() {}"));

    let (syntax, resolver, crate_scope_id) = bind_declarations(&source);
    let fn_body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);

    let scope = resolver.scopes.get_scope(fn_body_scope).unwrap();

    assert_eq!(scope.kind, ScopeKind::Function);
    assert_eq!(scope.parent, crate_scope_id);
}

#[test]
fn nested_function() {
    let mut source = Source::new();

    source.add_code(Code::validated_borrowed(
        "test",
        "fn outer() { fn inner() {} }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let outer_symbol = resolver.symbols.add_symbol("outer");
    let (_, outer_declaration) = resolver
        .declarations
        .find_declaration(outer_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        outer_declaration.definition,
        Definition::Function { .. }
    ));

    let outer_body_scope = find_function_body_scope(&syntax, &resolver, crate_scope_id);
    let inner_symbol = resolver.symbols.add_symbol("inner");
    let (_, inner_declaration) = resolver
        .declarations
        .find_declaration(inner_symbol, outer_body_scope, Visibility::Block)
        .unwrap();
    let Definition::Function {
        value_parameters,
        return_type_id,
        ..
    } = inner_declaration.definition
    else {
        panic!();
    };

    assert!(value_parameters.is_empty());
    assert_eq!(return_type_id, TypeId::UNIT);
    assert_eq!(inner_declaration.scope_id, outer_body_scope);
}
