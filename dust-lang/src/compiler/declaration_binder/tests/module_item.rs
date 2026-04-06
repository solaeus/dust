use crate::{
    resolver::{
        declarations::{Definition, ModuleKind, Visibility},
        scopes::ScopeKind,
    },
    source::{Source, SourceCode},
};

use super::{bind_declarations, cleanup_module_file, create_module_file};

#[test]
fn inline() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "mod foo {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        foo_declaration.definition,
        Definition::Module {
            public: false,
            kind: ModuleKind::Inline,
            ..
        }
    ));
}

#[test]
fn public_inline() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "pub mod foo {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        foo_declaration.definition,
        Definition::Module {
            public: true,
            kind: ModuleKind::Inline,
            ..
        }
    ));
}

#[test]
fn inline_creates_module_scope() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "mod foo {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module { inner_scope_id, .. } = foo_declaration.definition else {
        panic!();
    };

    let scope = resolver.scopes.get_scope(inner_scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Module);
    assert_eq!(scope.parent, crate_scope_id);
}

#[test]
fn inline_with_function() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "mod foo { fn bar() {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module { inner_scope_id, .. } = foo_declaration.definition else {
        panic!();
    };

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, inner_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        bar_declaration.definition,
        Definition::Function { .. }
    ));
}

#[test]
fn nested_inline() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "mod foo { mod bar {} }",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: foo_scope_id,
        ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, foo_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module {
        inner_scope_id: bar_scope_id,
        ..
    } = bar_declaration.definition
    else {
        panic!();
    };

    let bar_scope = resolver.scopes.get_scope(bar_scope_id).unwrap();

    assert_eq!(bar_scope.kind, ScopeKind::Module);
    assert_eq!(bar_scope.parent, foo_scope_id);
}

#[test]
fn multiple_inline() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "mod foo {} mod bar {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let foo_result =
        resolver
            .declarations
            .find_declaration(foo_symbol, crate_scope_id, Visibility::Module);
    assert!(foo_result.is_some());

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let bar_result =
        resolver
            .declarations
            .find_declaration(bar_symbol, crate_scope_id, Visibility::Module);
    assert!(bar_result.is_some());
}

#[test]
fn file() {
    let path = create_module_file("foo", "");
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "mod foo;"));
    source.add_file(SourceCode::file(path.clone()).unwrap());

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    cleanup_module_file(&path);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        foo_declaration.definition,
        Definition::Module {
            public: false,
            kind: ModuleKind::File { .. },
            ..
        }
    ));
}

#[test]
fn public_file() {
    let path = create_module_file("foo", "");
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "pub mod foo;"));
    source.add_file(SourceCode::file(path.clone()).unwrap());

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    cleanup_module_file(&path);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        foo_declaration.definition,
        Definition::Module {
            public: true,
            kind: ModuleKind::File { .. },
            ..
        }
    ));
}

#[test]
fn file_binds_contents() {
    let path = create_module_file("foo", "fn bar() {}");
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "mod foo;"));
    source.add_file(SourceCode::file(path.clone()).unwrap());

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    cleanup_module_file(&path);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Module { inner_scope_id, .. } = foo_declaration.definition else {
        panic!();
    };

    let bar_symbol = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .find_declaration(bar_symbol, inner_scope_id, Visibility::Module)
        .unwrap();

    assert!(matches!(
        bar_declaration.definition,
        Definition::Function { .. }
    ));
}
