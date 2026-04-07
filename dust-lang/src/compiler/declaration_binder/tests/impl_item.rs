use crate::{
    compiler::resolver::{
        declarations::{Definition, Visibility},
        types::TypeId,
    },
    source::{Source, SourceCode},
};

use super::{bind_declarations, bind_declarations_with_errors};

#[test]
fn with_method() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn bar() {} }",
    ));

    let (_syntax, mut resolver, _crate_scope_id) = bind_declarations(&source);
    let (_, impl_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, d)| matches!(d.definition, Definition::InherentImplementation { .. }))
        .unwrap();
    let Definition::InherentImplementation { declarations, .. } = impl_declaration.definition
    else {
        panic!();
    };

    assert_eq!(declarations.len(), 1);

    let method_ids = resolver
        .declarations
        .get_declaration_members(&declarations)
        .unwrap();
    let method = resolver
        .declarations
        .get_declaration(method_ids[0])
        .unwrap();
    let bar_symbol = resolver.symbols.add_symbol("bar");

    assert_eq!(method.symbol_id, bar_symbol);

    let Definition::Function {
        value_parameters,
        return_type_id,
        ..
    } = method.definition
    else {
        panic!();
    };

    assert!(value_parameters.is_empty());
    assert_eq!(return_type_id, TypeId::UNIT);
}

#[test]
fn methods_not_visible_at_module_scope() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn bar() {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("bar");
    let result =
        resolver
            .declarations
            .find_declaration(bar_symbol, crate_scope_id, Visibility::Module);

    assert!(result.is_none());
}

#[test]
fn impl_method_path_resolves() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn value() -> i32 { 42 } } fn main() { Foo::value(); }",
    ));

    let (_syntax, _resolver, _crate_scope_id, errors) = bind_declarations_with_errors(&source);

    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn impl_method_with_arguments_resolves() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn add(a: i32, b: i32) -> i32 { a + b } } fn main() { Foo::add(1, 2); }",
    ));

    let (_syntax, _resolver, _crate_scope_id, errors) = bind_declarations_with_errors(&source);

    assert!(errors.is_empty(), "{errors:#?}");
}
