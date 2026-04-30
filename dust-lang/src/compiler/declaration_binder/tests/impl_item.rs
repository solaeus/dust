use crate::{
    compiler::{
        resolver::{declarations::Definition, types::TypeId},
        tests::bind_declarations,
    },
    source::{Source, SourceCode, SourceCodeId},
    syntax::node::SyntaxKind,
};

#[test]
fn with_method() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn bar() {} }",
    ));

    let (syntax, mut resolver, _crate_scope_id) = bind_declarations(&source);
    let tree = syntax.get_tree(SourceCodeId::MAIN).unwrap();
    let impl_item = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::ImplItem)
        .unwrap();
    let impl_declaration_id = *resolver.get_declaration_binding(&impl_item.id).unwrap();
    let impl_declaration = resolver
        .declarations
        .get_declaration(impl_declaration_id)
        .unwrap();
    let Definition::InherentImplementation { declarations, .. } = impl_declaration.definition
    else {
        panic!();
    };

    assert_eq!(resolver.scopes.get_members(declarations.unwrap()).len(), 1);

    let method_entries = resolver.scopes.get_members(declarations.unwrap());
    let method = resolver
        .declarations
        .get_declaration(method_entries[0])
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

    assert_eq!(value_parameters, None);
    assert_eq!(return_type_id, TypeId::UNIT);
}

#[test]
fn methods_not_visible_at_module_scope() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn bar() {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("bar");
    let result = resolver
        .declarations
        .find_declaration_id(bar_symbol, crate_scope_id);

    assert!(result.is_none());
}

#[test]
fn impl_method_path_resolves() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn value() -> i32 { 42 } } fn main() { Foo::value(); }",
    ));

    let (_syntax, _resolver, _crate_scope_id) = bind_declarations(&source);
}

#[test]
fn impl_method_with_arguments_resolves() {
    let mut source = Source::new();

    source.add_code(SourceCode::validated_borrowed(
        "test",
        "struct Foo {} impl Foo { fn add(a: i32, b: i32) -> i32 { a + b } } fn main() { Foo::add(1, 2); }",
    ));

    let (_syntax, _resolver, _crate_scope_id) = bind_declarations(&source);
}
