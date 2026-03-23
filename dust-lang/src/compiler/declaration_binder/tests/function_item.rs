use crate::{
    resolver::{
        declarations::{Definition, Visibility},
        types::TypeId,
    },
    source::{Source, SourceFile},
};

use super::bind_declarations;

#[test]
fn minimal_function() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", "fn foo() {}"));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
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
    let params = resolver.types.get_type_members(value_parameters).unwrap();
    assert!(params.is_empty());
    assert_eq!(return_type_id, TypeId::UNIT);
    assert_eq!(foo_declaration.scope_id, crate_scope_id);
}

#[test]
fn fully_specified_function() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "pub fn foo<A, B>(x: i64, y: bool) -> i64 {}",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
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

    assert!(public);
    assert_eq!(return_type_id, TypeId::I_64);

    let type_param_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();
    assert_eq!(type_param_ids.len(), 2);
    for &id in type_param_ids {
        let decl = resolver.declarations.get_declaration(id).unwrap();
        assert!(matches!(decl.definition, Definition::TypeParameter));
    }

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");
    let first = resolver
        .declarations
        .get_declaration(type_param_ids[0])
        .unwrap();
    let second = resolver
        .declarations
        .get_declaration(type_param_ids[1])
        .unwrap();
    assert_eq!(first.symbol_id, a_symbol);
    assert_eq!(second.symbol_id, b_symbol);

    let value_param_types = resolver.types.get_type_members(value_parameters).unwrap();
    assert_eq!(value_param_types, &[TypeId::I_64, TypeId::BOOLEAN]);
}

#[test]
fn multiple_functions_in_same_scope() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "fn fib(n: i64) -> i64 {} fn main() -> i64 {}",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let fib_symbol = resolver.symbols.add_symbol("fib");
    let (_, fib_decl) = resolver
        .declarations
        .find_declaration(fib_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters: fib_params,
        return_type_id: fib_return,
        ..
    } = fib_decl.definition
    else {
        panic!();
    };
    let fib_param_types = resolver.types.get_type_members(fib_params).unwrap();
    assert_eq!(fib_param_types, &[TypeId::I_64]);
    assert_eq!(fib_return, TypeId::I_64);

    let main_symbol = resolver.symbols.add_symbol("main");
    let (_, main_decl) = resolver
        .declarations
        .find_declaration(main_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters: main_params,
        return_type_id: main_return,
        ..
    } = main_decl.definition
    else {
        panic!();
    };
    let main_param_types = resolver.types.get_type_members(main_params).unwrap();
    assert!(main_param_types.is_empty());
    assert_eq!(main_return, TypeId::I_64);
}

#[test]
fn parameters_not_visible_in_declaring_scope() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", "fn foo(x: i64) {}"));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
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
fn same_name_function_in_different_modules() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod a { fn foo() -> i64 {} } mod b { fn foo() -> bool {} }",
    ));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let a_symbol = resolver.symbols.add_symbol("a");
    let (_, a_decl) = resolver
        .declarations
        .find_declaration(a_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: a_scope,
        ..
    } = a_decl.definition
    else {
        panic!();
    };

    let b_symbol = resolver.symbols.add_symbol("b");
    let (_, b_decl) = resolver
        .declarations
        .find_declaration(b_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: b_scope,
        ..
    } = b_decl.definition
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
    source.add_file(SourceFile::validated_borrowed("test", "fn foo<A, B>() {}"));

    let (mut resolver, crate_scope_id) = bind_declarations(&source);
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

    let type_param_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();
    assert_eq!(type_param_ids.len(), 2);

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");

    let first = resolver
        .declarations
        .get_declaration(type_param_ids[0])
        .unwrap();
    let second = resolver
        .declarations
        .get_declaration(type_param_ids[1])
        .unwrap();

    assert_eq!(first.symbol_id, a_symbol);
    assert!(matches!(first.definition, Definition::TypeParameter));
    assert_eq!(second.symbol_id, b_symbol);
    assert!(matches!(second.definition, Definition::TypeParameter));
}
