use crate::{
    compiler::{
        declaration_binder::tests::find_function_body_scope,
        resolver::{declarations::Definition, scopes::ScopeKind, types::TypeId},
        tests::bind_declarations,
    },
    source::{Source, SourceCode},
};

#[test]
fn empty() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed("test", "fn foo() {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
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
        public,
        type_parameters,
        value_parameters,
        return_type_id,
    } = foo_declaration.definition
    else {
        panic!();
    };

    assert!(!public);
    assert_eq!(type_parameters, None);
    assert_eq!(value_parameters, None);
    assert_eq!(return_type_id, TypeId::UNIT);
}

#[test]
fn with_generics_parameters_and_return_type() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "pub fn foo<A, B>(x: i64, y: bool) -> i64 {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
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
        public,
        type_parameters,
        value_parameters,
        return_type_id,
    } = foo_declaration.definition
    else {
        panic!();
    };
    let type_parameter_entries = resolver
        .scopes
        .get_namespace_entries(type_parameters.unwrap());

    assert!(public);
    assert_eq!(return_type_id, TypeId::I_64);
    assert_eq!(type_parameter_entries.len(), 2);

    for &(_, id) in type_parameter_entries {
        let declaration = resolver.declarations.get_declaration(id).unwrap();

        assert!(matches!(declaration.definition, Definition::TypeParameter));
    }

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");
    let first = resolver
        .declarations
        .get_declaration(type_parameter_entries[0].1)
        .unwrap();
    let second = resolver
        .declarations
        .get_declaration(type_parameter_entries[1].1)
        .unwrap();

    assert_eq!(first.symbol_id, a_symbol);
    assert_eq!(second.symbol_id, b_symbol);

    let parameter_entries = resolver
        .scopes
        .get_namespace_entries(value_parameters.unwrap());

    assert_eq!(parameter_entries.len(), 2);

    let first_parameter = resolver
        .declarations
        .get_declaration(parameter_entries[0].1)
        .unwrap();
    let Definition::Local {
        type_id: first_type,
        ..
    } = first_parameter.definition
    else {
        panic!();
    };
    let second_parameter = resolver
        .declarations
        .get_declaration(parameter_entries[1].1)
        .unwrap();
    let Definition::Local {
        type_id: second_type,
        ..
    } = second_parameter.definition
    else {
        panic!();
    };

    assert_eq!(first_type, TypeId::I_64);
    assert_eq!(second_type, TypeId::BOOLEAN);
}

#[test]
fn parameters_not_visible_in_declaring_scope() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed("test", "fn foo(x: i64) {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let x_symbol = resolver.symbols.add_symbol("x");
    let result = resolver
        .declarations
        .find_declaration_id(x_symbol, crate_scope_id);

    assert!(
        result.is_none(),
        "parameter should not be visible in the declaring scope"
    );
}

#[test]
fn same_name_in_different_modules() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "mod a { fn foo() -> i64 {} } mod b { fn foo() -> bool {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    let a_symbol = resolver.symbols.add_symbol("a");
    let a_declaration_id = *resolver
        .declarations
        .find_declaration_id(a_symbol, crate_scope_id)
        .unwrap();
    let a_declaration = resolver
        .declarations
        .get_declaration(a_declaration_id)
        .unwrap();
    let Definition::Module {
        inner_scope_id: a_scope,
        ..
    } = a_declaration.definition
    else {
        panic!();
    };

    let b_symbol = resolver.symbols.add_symbol("b");
    let b_declaration_id = *resolver
        .declarations
        .find_declaration_id(b_symbol, crate_scope_id)
        .unwrap();
    let b_declaration = resolver
        .declarations
        .get_declaration(b_declaration_id)
        .unwrap();
    let Definition::Module {
        inner_scope_id: b_scope,
        ..
    } = b_declaration.definition
    else {
        panic!();
    };

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let a_foo_id = *resolver
        .declarations
        .find_declaration_id(foo_symbol, a_scope.unwrap())
        .unwrap();
    let a_foo = resolver.declarations.get_declaration(a_foo_id).unwrap();
    let Definition::Function {
        return_type_id: a_return,
        ..
    } = a_foo.definition
    else {
        panic!();
    };

    assert_eq!(a_return, TypeId::I_64);

    let b_foo_id = *resolver
        .declarations
        .find_declaration_id(foo_symbol, b_scope.unwrap())
        .unwrap();
    let b_foo = resolver.declarations.get_declaration(b_foo_id).unwrap();
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

    source.add_code(SourceCode::validated_borrowed("test", "fn foo<A, B>() {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
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
        .get_namespace_entries(type_parameters.unwrap());

    assert_eq!(type_parameter_entries.len(), 2);

    let a_symbol = resolver.symbols.add_symbol("A");
    let b_symbol = resolver.symbols.add_symbol("B");

    let first = resolver
        .declarations
        .get_declaration(type_parameter_entries[0].1)
        .unwrap();
    let second = resolver
        .declarations
        .get_declaration(type_parameter_entries[1].1)
        .unwrap();

    assert_eq!(first.symbol_id, a_symbol);
    assert!(matches!(first.definition, Definition::TypeParameter));
    assert_eq!(second.symbol_id, b_symbol);
    assert!(matches!(second.definition, Definition::TypeParameter));
}

#[test]
fn value_parameter_declarations() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn foo(x: i64, y: bool) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
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
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let x_symbol = resolver.symbols.add_symbol("x");
    let x_declaration_id = *resolver
        .declarations
        .find_declaration_id(x_symbol, value_parameters.unwrap())
        .unwrap();
    let x_declaration = resolver
        .declarations
        .get_declaration(x_declaration_id)
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
    let y_declaration_id = *resolver
        .declarations
        .find_declaration_id(y_symbol, value_parameters.unwrap())
        .unwrap();
    let y_declaration = resolver
        .declarations
        .get_declaration(y_declaration_id)
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

    source.add_code(SourceCode::validated_borrowed("test", "fn foo() {}"));

    let (_syntax, resolver, crate_scope_id) = bind_declarations(&source);
    let fn_body_scope = find_function_body_scope(&resolver, crate_scope_id);

    let scope = resolver.scopes.get_scope(fn_body_scope);

    assert_eq!(scope.kind, ScopeKind::Block);
    assert_eq!(scope.parent, Some(crate_scope_id));
}

#[test]
fn nested_function() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "fn outer() { fn inner() {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let outer_symbol = resolver.symbols.add_symbol("outer");
    let outer_declaration_id = *resolver
        .declarations
        .find_declaration_id(outer_symbol, crate_scope_id)
        .unwrap();
    let outer_declaration = resolver
        .declarations
        .get_declaration(outer_declaration_id)
        .unwrap();

    assert!(matches!(
        outer_declaration.definition,
        Definition::Function { .. }
    ));

    let outer_body_scope = find_function_body_scope(&resolver, crate_scope_id);
    let inner_symbol = resolver.symbols.add_symbol("inner");
    let inner_declaration_id = *resolver
        .declarations
        .find_declaration_id(inner_symbol, outer_body_scope)
        .unwrap();
    let inner_declaration = resolver
        .declarations
        .get_declaration(inner_declaration_id)
        .unwrap();
    let Definition::Function {
        value_parameters,
        return_type_id,
        ..
    } = inner_declaration.definition
    else {
        panic!();
    };

    assert_eq!(value_parameters, None);
    assert_eq!(return_type_id, TypeId::UNIT);
    assert_eq!(inner_declaration.scope_id, outer_body_scope);
}
