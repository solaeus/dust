use crate::{
    resolver::declarations::{Definition, Visibility},
    source::{Source, SourceFile},
};

use super::bind_declarations;

#[test]
fn module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", "mod foo {} use foo;"));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        use_declaration.definition,
        Definition::Use {
            public: false,
            ..
        }
    ));
}

#[test]
fn public_module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo {} pub use foo;",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        use_declaration.definition,
        Definition::Use {
            public: true,
            ..
        }
    ));
}

#[test]
fn resolves_to_module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", "mod foo {} use foo;"));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Use { item, .. } = use_declaration.definition else {
        panic!();
    };

    let target = resolver.declarations.get_declaration(item).unwrap();

    assert!(matches!(target.definition, Definition::Module { .. }));
}

#[test]
fn function_from_module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo { fn bar() {} } use foo::bar;",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Use { item, .. } = use_declaration.definition else {
        panic!();
    };

    let target = resolver.declarations.get_declaration(item).unwrap();

    assert!(matches!(target.definition, Definition::Function { .. }));
}

#[test]
fn struct_from_module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo { struct Bar; } use foo::Bar;",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Use { item, .. } = use_declaration.definition else {
        panic!();
    };

    let target = resolver.declarations.get_declaration(item).unwrap();

    assert!(matches!(target.definition, Definition::StructType { .. }));
}

#[test]
fn from_nested_module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo { mod bar { fn baz() {} } } use foo::bar::baz;",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let baz_symbol = resolver.symbols.add_symbol("baz");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(baz_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Use { item, .. } = use_declaration.definition else {
        panic!();
    };

    let target = resolver.declarations.get_declaration(item).unwrap();

    assert!(matches!(target.definition, Definition::Function { .. }));
}

#[test]
fn nested_module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo { mod bar {} } use foo::bar;",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Use { item, .. } = use_declaration.definition else {
        panic!();
    };

    let target = resolver.declarations.get_declaration(item).unwrap();

    assert!(matches!(target.definition, Definition::Module { .. }));
}

#[test]
fn inside_module() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo { fn bar() {} } mod baz { use foo::bar; }",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let baz_symbol = resolver.symbols.add_symbol("baz");
    let (_, baz_declaration) = resolver
        .declarations
        .find_declaration(baz_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module { inner_scope_id, .. } = baz_declaration.definition else {
        panic!();
    };

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, inner_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Use { item, .. } = use_declaration.definition else {
        panic!();
    };

    let target = resolver.declarations.get_declaration(item).unwrap();

    assert!(matches!(target.definition, Definition::Function { .. }));
}

#[test]
fn public_function() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo { fn bar() {} } pub use foo::bar;",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let (_, use_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        use_declaration.definition,
        Definition::Use {
            public: true,
            ..
        }
    ));
}

#[test]
fn multiple() {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed(
        "test",
        "mod foo { fn bar() {} fn baz() {} } use foo::bar; use foo::baz;",
    ));
    let (mut resolver, crate_scope_id) = bind_declarations(&source);

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let bar_result =
        resolver
            .declarations
            .find_declaration(bar_symbol, crate_scope_id, Visibility::Module);
    assert!(bar_result.is_some());

    let baz_symbol = resolver.symbols.add_symbol("baz");
    let baz_result =
        resolver
            .declarations
            .find_declaration(baz_symbol, crate_scope_id, Visibility::Module);
    assert!(baz_result.is_some());
}
