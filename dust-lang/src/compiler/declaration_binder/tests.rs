use smallvec::SmallVec;

use crate::{
    lexer::Lexer,
    parser::Parser,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind, ModuleKind},
        scope_graph::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxVisitor},
};

use super::DeclarationBinder;

fn bind_declarations(source_code: &str) -> Resolver {
    let mut source = Source::new();

    source.add_file(SourceFile::validated("test", source_code));

    let lexer = Lexer::from_utf8(source_code);
    let parser = Parser::new(SourceFileId::MAIN, lexer);
    let parse_result = parser.parse();

    let mut syntax = Syntax::new(source.file_count());

    syntax.add_tree(parse_result.syntax_tree).unwrap();

    let mut resolver = Resolver::new();
    let program_scope_id = resolver.scopes.add_scope(Scope {
        kind: ScopeKind::Crate,
        parent: ScopeId::NONE,
        modules: SmallVec::new(),
        imports: SmallVec::new(),
    });

    let main_root = syntax.get_tree(SourceFileId::MAIN).unwrap().root().unwrap();

    let mut errors = Vec::new();
    let mut declaration_binder = DeclarationBinder::new(
        &source,
        &syntax,
        &mut resolver,
        &mut errors,
        program_scope_id,
    );

    match declaration_binder.visit_root(main_root) {
        Ok(()) => {}
        Err(error) => errors.push(error),
    }

    assert!(errors.is_empty(), "{errors:#?}");

    resolver
}

fn find_declaration(
    resolver: &mut Resolver,
    name: &str,
) -> Option<(DeclarationId, DeclarationKind)> {
    let symbol_id = resolver.symbols.add_symbol(name);

    resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == symbol_id)
        .map(|(id, declaration)| (id, declaration.kind))
}

#[test]
fn let_creates_local_declaration() {
    let mut resolver = bind_declarations("fn main() { let x = 42; }");

    let (_, x_kind) = find_declaration(&mut resolver, "x").unwrap();

    assert!(matches!(x_kind, DeclarationKind::Local { shadowed: None }));
}

#[test]
fn let_creates_non_public_declaration() {
    let mut resolver = bind_declarations("fn main() { let x = 42; }");

    let x_symbol_id = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == x_symbol_id)
        .unwrap();

    assert!(!x_declaration.is_public);
}

#[test]
fn let_shadowing_across_scopes_links_to_previous_declaration() {
    let mut resolver = bind_declarations("fn main() { let x = 1; { let x = 2; } }");

    let x_symbol_id = resolver.symbols.add_symbol("x");
    let x_declarations: Vec<_> = resolver
        .declarations
        .iter()
        .filter(|(_, declaration)| declaration.symbol_id == x_symbol_id)
        .collect();

    assert_eq!(x_declarations.len(), 2);

    let (first_x_declaration_id, first_x_declaration) = x_declarations[0];
    let (_, second_x_declaration) = x_declarations[1];

    assert!(matches!(
        first_x_declaration.kind,
        DeclarationKind::Local { shadowed: None }
    ));
    assert!(matches!(
        second_x_declaration.kind,
        DeclarationKind::Local { shadowed: Some(shadowed_id) }
            if shadowed_id == first_x_declaration_id
    ));
}

#[test]
fn function_item_creates_function_declaration() {
    let mut resolver = bind_declarations("fn foo() {}");

    let (_, foo_kind) = find_declaration(&mut resolver, "foo").unwrap();

    assert_eq!(foo_kind, DeclarationKind::Function);
}

#[test]
fn function_item_is_not_public() {
    let mut resolver = bind_declarations("fn foo() {}");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(!foo_declaration.is_public);
}

#[test]
fn public_function_item_is_public() {
    let mut resolver = bind_declarations("pub fn foo() {}");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(foo_declaration.is_public);
}

#[test]
fn function_item_is_declared_in_project_scope() {
    let mut resolver = bind_declarations("fn foo() {}");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    let declaration_scope = resolver.scopes.get_scope(foo_declaration.scope_id).unwrap();

    assert_eq!(declaration_scope.kind, ScopeKind::Crate);
}

#[test]
fn inline_module_creates_module_declaration() {
    let mut resolver = bind_declarations("mod foo { fn bar() {} }");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(matches!(
        foo_declaration.kind,
        DeclarationKind::Module {
            kind: ModuleKind::Inline,
            ..
        }
    ));
}

#[test]
fn inline_module_creates_module_scope() {
    let mut resolver = bind_declarations("mod foo { fn bar() {} }");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    let inner_scope_id = match foo_declaration.kind {
        DeclarationKind::Module { inner_scope_id, .. } => inner_scope_id,
        other => panic!("expected Module declaration, got {other:?}"),
    };

    let inner_scope = resolver.scopes.get_scope(inner_scope_id).unwrap();

    assert_eq!(inner_scope.kind, ScopeKind::Module);
}

#[test]
fn inline_module_items_are_declared_in_module_scope() {
    let mut resolver = bind_declarations("mod foo { fn bar() {} }");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    let module_scope_id = match foo_declaration.kind {
        DeclarationKind::Module { inner_scope_id, .. } => inner_scope_id,
        other => panic!("expected Module declaration, got {other:?}"),
    };

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .unwrap();

    assert_eq!(bar_declaration.scope_id, module_scope_id);
}

#[test]
fn struct_creates_type_declaration() {
    let mut resolver = bind_declarations("struct Foo { x: int, y: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    assert!(matches!(foo_kind, DeclarationKind::Type { .. }));
}

#[test]
fn struct_field_count_matches_definition() {
    let mut resolver = bind_declarations("struct Foo { x: int, y: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    let members = match foo_kind {
        DeclarationKind::Type { members, .. } => members,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(members.count, 2);
}

#[test]
fn struct_fields_reference_parent() {
    let mut resolver = bind_declarations("struct Foo { x: int }");

    let (foo_declaration_id, _) = find_declaration(&mut resolver, "Foo").unwrap();

    let (_, x_kind) = find_declaration(&mut resolver, "x").unwrap();

    let parent_id = match x_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration for field, got {other:?}"),
    };

    assert_eq!(parent_id, Some(foo_declaration_id));
}

#[test]
fn struct_has_no_parent() {
    let mut resolver = bind_declarations("struct Foo { x: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    let parent_id = match foo_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent_id, None);
}

#[test]
fn enum_creates_type_declaration() {
    let mut resolver = bind_declarations("enum Color { Red, Green, Blue }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    assert!(matches!(color_kind, DeclarationKind::Type { .. }));
}

#[test]
fn enum_variant_count_matches_definition() {
    let mut resolver = bind_declarations("enum Color { Red, Green, Blue }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    let members = match color_kind {
        DeclarationKind::Type { members, .. } => members,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(members.count, 3);
}

#[test]
fn enum_variants_reference_parent() {
    let mut resolver = bind_declarations("enum Color { Red }");

    let (color_declaration_id, _) = find_declaration(&mut resolver, "Color").unwrap();

    let (_, red_kind) = find_declaration(&mut resolver, "Red").unwrap();

    let parent_id = match red_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration for variant, got {other:?}"),
    };

    assert_eq!(parent_id, Some(color_declaration_id));
}

#[test]
fn enum_has_no_parent() {
    let mut resolver = bind_declarations("enum Color { Red }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    let parent_id = match color_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent_id, None);
}

#[test]
fn nested_blocks_create_scope_chain() {
    let mut resolver = bind_declarations("fn main() { let a = 1; { let b = 2; { let c = 3; } } }");

    let a_symbol_id = resolver.symbols.add_symbol("a");
    let b_symbol_id = resolver.symbols.add_symbol("b");
    let c_symbol_id = resolver.symbols.add_symbol("c");

    let a_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == a_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let b_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == b_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let c_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == c_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let b_scope = resolver.scopes.get_scope(b_scope_id).unwrap();
    let c_scope = resolver.scopes.get_scope(c_scope_id).unwrap();

    assert_eq!(b_scope.parent, a_scope_id);
    assert_eq!(c_scope.parent, b_scope_id);
}

#[test]
fn function_expression_body_is_in_function_scope() {
    let mut resolver = bind_declarations("fn main() { let f = () { let y = 1; }; }");

    let y_symbol_id = resolver.symbols.add_symbol("y");
    let y_scope_id = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == y_symbol_id)
        .unwrap()
        .1
        .scope_id;

    let y_scope = resolver.scopes.get_scope(y_scope_id).unwrap();
    let block_scope = resolver.scopes.get_scope(y_scope.parent).unwrap();
    let function_scope = resolver.scopes.get_scope(block_scope.parent).unwrap();

    assert_eq!(function_scope.kind, ScopeKind::Function);
}

#[test]
fn path_expression_resolves_to_local_declaration() {
    let mut resolver = bind_declarations("fn main() { let x = 1; x }");

    let (x_declaration_id, _) = find_declaration(&mut resolver, "x").unwrap();

    assert!(matches!(
        resolver
            .declarations
            .get_declaration(x_declaration_id)
            .unwrap()
            .kind,
        DeclarationKind::Local { .. }
    ));
}

#[test]
fn qualified_path_resolves_through_module() {
    let mut resolver = bind_declarations("mod foo { pub fn bar() {} } fn main() { foo::bar }");

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .unwrap();

    assert_eq!(bar_declaration.kind, DeclarationKind::Function);
}
