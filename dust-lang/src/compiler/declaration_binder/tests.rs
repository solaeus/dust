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
    syntax::{Syntax, SyntaxKind, SyntaxVisitor},
};

use super::DeclarationBinder;

fn bind_declarations(source_code: &str) -> (Syntax, Resolver) {
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

    (syntax, resolver)
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
    let (_syntax, mut resolver) = bind_declarations("fn main() { let x = 42; }");

    let (_, x_kind) = find_declaration(&mut resolver, "x").unwrap();

    assert!(matches!(x_kind, DeclarationKind::Local));
}

#[test]
fn let_creates_non_public_declaration() {
    let (_syntax, mut resolver) = bind_declarations("fn main() { let x = 42; }");

    let x_symbol_id = resolver.symbols.add_symbol("x");
    let (_, x_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == x_symbol_id)
        .unwrap();

    assert!(!x_declaration.is_public);
}

#[test]
fn function_item_creates_function_declaration() {
    let (_syntax, mut resolver) = bind_declarations("fn foo() {}");

    let (_, foo_kind) = find_declaration(&mut resolver, "foo").unwrap();

    assert_eq!(foo_kind, DeclarationKind::Function);
}

#[test]
fn function_item_is_not_public() {
    let (_syntax, mut resolver) = bind_declarations("fn foo() {}");

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
    let (_syntax, mut resolver) = bind_declarations("pub fn foo() {}");

    let foo_symbol_id = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(foo_declaration.is_public);
}

#[test]
fn function_item_declares_function() {
    let (_syntax, mut resolver) = bind_declarations("fn foo() {}");

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
fn inline_module_declares_module() {
    let (_syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

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
    let (_syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

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
    let (_syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");

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
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int, y: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    assert!(matches!(foo_kind, DeclarationKind::Type { .. }));
}

#[test]
fn struct_field_count_matches_definition() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int, y: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    let members = match foo_kind {
        DeclarationKind::Type { members, .. } => members,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(members.count, 2);
}

#[test]
fn struct_fields_reference_parent() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int }");

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
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int }");

    let (_, foo_kind) = find_declaration(&mut resolver, "Foo").unwrap();

    let parent_id = match foo_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent_id, None);
}

#[test]
fn enum_creates_type_declaration() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red, Green, Blue }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    assert!(matches!(color_kind, DeclarationKind::Type { .. }));
}

#[test]
fn enum_variant_count_matches_definition() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red, Green, Blue }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    let members = match color_kind {
        DeclarationKind::Type { members, .. } => members,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(members.count, 3);
}

#[test]
fn enum_variants_reference_parent() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red }");

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
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red }");

    let (_, color_kind) = find_declaration(&mut resolver, "Color").unwrap();

    let parent_id = match color_kind {
        DeclarationKind::Type { parent, .. } => parent,
        other => panic!("expected Type declaration, got {other:?}"),
    };

    assert_eq!(parent_id, None);
}

#[test]
fn nested_blocks_create_scope_chain() {
    let (_syntax, mut resolver) =
        bind_declarations("fn main() { let a = 1; { let b = 2; { let c = 3; } } }");

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
    let (_syntax, mut resolver) = bind_declarations("fn main() { let f = () { let y = 1; }; }");

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
    let (_syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x }");

    let (x_declaration_id, _) = find_declaration(&mut resolver, "x").unwrap();

    assert!(matches!(
        resolver
            .declarations
            .get_declaration(x_declaration_id)
            .unwrap()
            .kind,
        DeclarationKind::Local
    ));
}

#[test]
fn qualified_path_resolves_through_module() {
    let (_syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } fn main() { foo::bar }");

    let bar_symbol_id = resolver.symbols.add_symbol("bar");
    let (_, bar_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == bar_symbol_id)
        .unwrap();

    assert_eq!(bar_declaration.kind, DeclarationKind::Function);
}

#[test]
fn let_statement_binds_identifier_to_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 42; }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let let_stmt = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::LetStatement)
        .unwrap();
    let (path, _) = let_stmt.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&path.id).unwrap(), x_id);
}

#[test]
fn function_item_binds_name_to_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn foo() {}");
    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let func = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::FunctionItem)
        .unwrap();
    let (name, _) = func.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), foo_id);
}

#[test]
fn struct_name_binds_to_type_declaration() {
    let (syntax, mut resolver) = bind_declarations("struct Foo { x: int }");
    let (foo_id, _) = find_declaration(&mut resolver, "Foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructItem)
        .unwrap();
    let (name, _) = item.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), foo_id);
}

#[test]
fn struct_field_name_binds_to_field_declaration() {
    let (syntax, mut resolver) = bind_declarations("struct Foo { x: int }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let fields = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructFieldsDeclaration)
        .unwrap();
    let name = fields.children().unwrap().next().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), x_id);
}

#[test]
fn enum_name_binds_to_type_declaration() {
    let (syntax, mut resolver) = bind_declarations("enum Color { Red }");
    let (color_id, _) = find_declaration(&mut resolver, "Color").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::EnumItem)
        .unwrap();
    let mut children = item.children().unwrap();
    let name = children.next().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&name.id).unwrap(),
        color_id
    );
}

#[test]
fn enum_variant_name_binds_to_variant_declaration() {
    let (syntax, mut resolver) = bind_declarations("enum Color { Red }");
    let (red_id, _) = find_declaration(&mut resolver, "Red").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let variant = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::EnumVariant)
        .unwrap();
    let name = variant.child().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&name.id).unwrap(), red_id);
}

#[test]
fn path_expression_binds_to_referenced_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let path_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::PathExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&path_expr.id).unwrap(),
        x_id
    );
}

#[test]
fn inline_module_item_binds_to_module_declaration() {
    let (syntax, mut resolver) = bind_declarations("mod foo { fn bar() {} }");
    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let module = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ModuleItem)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&module.id).unwrap(),
        foo_id
    );
}

#[test]
fn block_expression_binds_to_block_scope() {
    let (syntax, resolver) = bind_declarations("fn main() { let x = 1; { let y = 2; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks = tree
        .iter()
        .filter(|reader| reader.kind() == SyntaxKind::BlockExpression)
        .collect::<Vec<_>>();
    let inner_block = blocks.last().unwrap();

    let scope_id = resolver.get_scope_binding(&inner_block.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Block);
}

#[test]
fn module_body_binds_to_module_scope() {
    let (syntax, resolver) = bind_declarations("mod foo { fn bar() {} }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let body = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ModuleBody)
        .unwrap();

    let scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Module);
}

#[test]
fn function_expression_body_binds_to_function_scope() {
    let (syntax, resolver) = bind_declarations("fn main() { let f = () { let y = 1; }; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let fn_exprs = tree
        .iter()
        .filter(|reader| reader.kind() == SyntaxKind::FunctionExpression)
        .collect::<Vec<_>>();
    let inner_fn = fn_exprs.last().unwrap();
    let (_, body) = inner_fn.binary_children().unwrap();

    let scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();
    let parent_scope = resolver.scopes.get_scope(scope.parent).unwrap();

    assert_eq!(parent_scope.kind, ScopeKind::Function);
}

#[test]
fn reassignment_resolves_path() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x = 2; }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let reassignment = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ReassignmentStatement)
        .unwrap();
    let (path, _) = reassignment.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&path.id).unwrap(), x_id);
}

#[test]
fn binary_assignment_resolves_path() {
    let (syntax, mut resolver) = bind_declarations("fn main() { let x = 1; x += 2; }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let assignment = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AdditionAssignmentStatement)
        .unwrap();
    let (path, _) = assignment.binary_children().unwrap();

    assert_eq!(*resolver.get_declaration_binding(&path.id).unwrap(), x_id);
}

#[test]
fn if_expression_then_block_creates_scope() {
    let (syntax, resolver) = bind_declarations("fn main() { if true { let a = 1; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let if_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::IfExpression)
        .unwrap();
    let mut children = if_expr.children().unwrap();
    let _condition = children.next().unwrap();
    let then_block = children.next().unwrap();

    let scope_id = resolver.get_scope_binding(&then_block.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Block);
}

#[test]
fn if_else_creates_scopes_for_both_branches() {
    let (syntax, resolver) =
        bind_declarations("fn main() { if true { let a = 1; } else { let b = 2; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let if_expression = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::IfExpression)
        .unwrap();
    let mut children = if_expression.children().unwrap();
    let _condition = children.next().unwrap();
    let then_block = children.next().unwrap();
    let else_block = children.next().unwrap();

    let then_scope_id = resolver.get_scope_binding(&then_block.id).unwrap();
    let then_scope = resolver.scopes.get_scope(*then_scope_id).unwrap();
    assert_eq!(then_scope.kind, ScopeKind::Block);

    let else_scope_id = resolver.get_scope_binding(&else_block.id).unwrap();
    let else_scope = resolver.scopes.get_scope(*else_scope_id).unwrap();
    assert_eq!(else_scope.kind, ScopeKind::Block);
}

#[test]
fn while_body_creates_block_scope() {
    let (syntax, resolver) = bind_declarations("fn main() { while true { let x = 1; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let while_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::WhileExpression)
        .unwrap();
    let (_condition, body) = while_expr.binary_children().unwrap();

    let scope_id = resolver.get_scope_binding(&body.id).unwrap();
    let scope = resolver.scopes.get_scope(*scope_id).unwrap();

    assert_eq!(scope.kind, ScopeKind::Block);
}

#[test]
fn call_expression_resolves_callee() {
    let (syntax, mut resolver) = bind_declarations("fn foo() {} fn main() { foo() }");
    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::CallExpression)
        .unwrap();
    let (callee, _) = call.binary_children().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&callee.id).unwrap(),
        foo_id
    );
}

#[test]
fn function_parameters_create_local_declarations() {
    let (_syntax, mut resolver) = bind_declarations("fn foo(x: int, y: bool) {}");

    let (_, x_kind) = find_declaration(&mut resolver, "x").unwrap();
    let (_, y_kind) = find_declaration(&mut resolver, "y").unwrap();

    assert!(matches!(x_kind, DeclarationKind::Local));
    assert!(matches!(y_kind, DeclarationKind::Local));
}

#[test]
fn function_parameter_binds_to_declaration() {
    let (syntax, mut resolver) = bind_declarations("fn foo(x: int) {}");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let value_params = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ValueParameters)
        .unwrap();
    let param_name = value_params.children().unwrap().next().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&param_name.id).unwrap(),
        x_id
    );
}

#[test]
fn struct_expression_resolves_field_paths() {
    let (syntax, mut resolver) =
        bind_declarations("struct Foo { x: int } fn main() { Foo { x: 1 } }");
    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let struct_expr = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StructExpression)
        .unwrap();
    let (_path, fields) = struct_expr.binary_children().unwrap();
    let field = fields.children().unwrap().next().unwrap();
    let (field_path, _) = field.binary_children().unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&field_path.id).unwrap(),
        x_id
    );
}

#[test]
fn use_item_binds_to_declaration() {
    let (syntax, mut resolver) =
        bind_declarations("mod foo { pub fn bar() {} } use foo::bar; fn main() { bar }");
    let (bar_id, _) = find_declaration(&mut resolver, "bar").unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let use_item = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::UseItem)
        .unwrap();

    assert_eq!(
        *resolver.get_declaration_binding(&use_item.id).unwrap(),
        bar_id
    );
}

#[test]
fn struct_item_is_not_public() {
    let (_syntax, mut resolver) = bind_declarations("struct Foo { x: int }");

    let foo_symbol_id = resolver.symbols.add_symbol("Foo");
    let (_, foo_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == foo_symbol_id)
        .unwrap();

    assert!(!foo_declaration.is_public);
}

#[test]
fn enum_item_is_not_public() {
    let (_syntax, mut resolver) = bind_declarations("enum Color { Red }");

    let color_symbol_id = resolver.symbols.add_symbol("Color");
    let (_, color_declaration) = resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == color_symbol_id)
        .unwrap();

    assert!(!color_declaration.is_public);
}
