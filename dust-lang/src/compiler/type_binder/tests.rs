use smallvec::SmallVec;

use crate::{
    lexer::Lexer,
    parser::Parser,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind},
        scope_graph::{Scope, ScopeId, ScopeKind},
        type_graph::{TypeId, TypeNode},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxKind, SyntaxVisitor},
};

use crate::compiler::declaration_binder::DeclarationBinder;

use super::TypeBinder;

fn bind_types(source_code: &str) -> (Syntax, Resolver) {
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

    {
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
    }

    assert!(errors.is_empty(), "declaration binding errors: {errors:#?}");

    {
        let main_root = syntax.get_tree(SourceFileId::MAIN).unwrap().root().unwrap();
        let mut type_binder = TypeBinder::new(&syntax, &mut resolver, &mut errors);

        match type_binder.visit_root(main_root) {
            Ok(()) => {}
            Err(error) => errors.push(error),
        }
    }

    assert!(errors.is_empty(), "type binding errors: {errors:#?}");

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
fn integer_expression_has_integer_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { 42 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::IntegerExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn float_expression_has_float_type() {
    let (syntax, resolver) = bind_types("fn main() -> float { 42.0 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::FloatExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::FLOAT);
}

#[test]
fn boolean_expression_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { true }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::BooleanExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn string_expression_has_string_type() {
    let (syntax, resolver) = bind_types("fn main() -> str { \"hello\" }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::StringExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::STRING
    );
}

#[test]
fn character_expression_has_character_type() {
    let (syntax, resolver) = bind_types("fn main() -> char { 'a' }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::CharacterExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::CHARACTER
    );
}

#[test]
fn byte_expression_has_byte_type() {
    let (syntax, resolver) = bind_types("fn main() -> byte { 0x2A }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ByteExpression)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::BYTE);
}

#[test]
fn let_statement_has_unit_type() {
    let (syntax, resolver) = bind_types("fn main() { let x = 42; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::LetStatement)
        .unwrap();

    assert_eq!(*resolver.get_type_binding(&node.id).unwrap(), TypeId::UNIT);
}

#[test]
fn let_declaration_gets_expression_type() {
    let (_syntax, mut resolver) = bind_types("fn main() { let x = 42; }");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    assert_eq!(x_type, TypeId::INTEGER);
}

#[test]
fn let_with_type_annotation_sets_declaration_type() {
    let (_syntax, mut resolver) = bind_types("fn main() { let x: int = 42; }");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    assert_eq!(x_type, TypeId::INTEGER);
}

#[test]
fn function_item_creates_function_type() {
    let (_syntax, mut resolver) = bind_types("fn foo() {}");

    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();
    let foo_type_id = *resolver.declarations.get_declaration_type(&foo_id).unwrap();
    let foo_type = *resolver.types.get_type(foo_type_id).unwrap();

    assert!(matches!(foo_type, TypeNode::Function { .. }));
}

#[test]
fn function_with_return_type() {
    let (_syntax, mut resolver) = bind_types("fn foo() -> int { 42 }");

    let (foo_id, _) = find_declaration(&mut resolver, "foo").unwrap();
    let foo_type_id = *resolver.declarations.get_declaration_type(&foo_id).unwrap();
    let foo_type = *resolver.types.get_type(foo_type_id).unwrap();

    let return_type_id = match foo_type {
        TypeNode::Function { return_type_id, .. } => return_type_id,
        other => panic!("expected Function type, got {other:?}"),
    };

    assert_eq!(return_type_id, TypeId::INTEGER);
}

#[test]
fn function_with_parameters_creates_typed_parameters() {
    let (_syntax, mut resolver) = bind_types("fn foo(x: int, y: bool) {}");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    let (y_id, _) = find_declaration(&mut resolver, "y").unwrap();
    let y_type = *resolver.declarations.get_declaration_type(&y_id).unwrap();

    assert_eq!(x_type, TypeId::INTEGER);
    assert_eq!(y_type, TypeId::BOOLEAN);
}

#[test]
fn function_expression_has_function_type_binding() {
    let (syntax, resolver) = bind_types("fn main() { let f = fn(x: int) -> int { x }; }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let fn_exprs: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::FunctionExpression)
        .collect();
    let inner_fn = fn_exprs.last().unwrap();

    let type_id = *resolver.get_type_binding(&inner_fn.id).unwrap();
    let type_node = *resolver.types.get_type(type_id).unwrap();

    assert!(matches!(type_node, TypeNode::Function { .. }));
}

#[test]
fn struct_item_creates_struct_type() {
    let (_syntax, mut resolver) = bind_types("struct Foo { x: int }");

    let (foo_id, _) = find_declaration(&mut resolver, "Foo").unwrap();
    let foo_type_id = *resolver.declarations.get_declaration_type(&foo_id).unwrap();
    let foo_type = *resolver.types.get_type(foo_type_id).unwrap();

    assert!(matches!(foo_type, TypeNode::Struct { .. }));
}

#[test]
fn struct_field_gets_declared_type() {
    let (_syntax, mut resolver) = bind_types("struct Foo { x: int }");

    let (x_id, _) = find_declaration(&mut resolver, "x").unwrap();
    let x_type = *resolver.declarations.get_declaration_type(&x_id).unwrap();

    assert_eq!(x_type, TypeId::INTEGER);
}

#[test]
fn enum_item_creates_enum_type() {
    let (_syntax, mut resolver) = bind_types("enum Color { Red }");

    let (color_id, _) = find_declaration(&mut resolver, "Color").unwrap();
    let color_type_id = *resolver
        .declarations
        .get_declaration_type(&color_id)
        .unwrap();
    let color_type = *resolver.types.get_type(color_type_id).unwrap();

    assert!(matches!(color_type, TypeNode::Enum { .. }));
}

#[test]
fn empty_block_has_unit_type() {
    let (syntax, resolver) = bind_types("fn main() { {} }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::BlockExpression)
        .collect();
    let inner_block = blocks.last().unwrap();

    assert_eq!(
        *resolver.get_type_binding(&inner_block.id).unwrap(),
        TypeId::UNIT
    );
}

#[test]
fn block_with_expression_has_expression_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { { 42 } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::BlockExpression)
        .collect();
    let inner_block = blocks.last().unwrap();

    assert_eq!(
        *resolver.get_type_binding(&inner_block.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn block_with_statement_has_unit_type() {
    let (syntax, resolver) = bind_types("fn main() { { let x = 42; } }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let blocks: Vec<_> = tree
        .iter()
        .filter(|node| node.kind() == SyntaxKind::BlockExpression)
        .collect();
    let inner_block = blocks.last().unwrap();

    assert_eq!(
        *resolver.get_type_binding(&inner_block.id).unwrap(),
        TypeId::UNIT
    );
}

#[test]
fn addition_expression_has_operand_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { 1 + 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AdditionExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn comparison_expression_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { 1 == 2 }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::EqualExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn logical_and_has_boolean_type() {
    let (syntax, resolver) = bind_types("fn main() -> bool { true && false }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::AndExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::BOOLEAN
    );
}

#[test]
fn path_expression_has_declaration_type() {
    let (syntax, resolver) = bind_types("fn main() -> int { let x = 42; x }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::PathExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}

#[test]
fn list_expression_creates_list_type() {
    let (syntax, resolver) = bind_types("fn main() -> [int] { [1, 2, 3] }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::ListExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&node.id).unwrap();
    let type_node = *resolver.types.get_type(type_id).unwrap();

    let element_type = match type_node {
        TypeNode::List { element_type } => element_type,
        other => panic!("expected List type, got {other:?}"),
    };

    assert_eq!(element_type, TypeId::INTEGER);
}

#[test]
fn call_expression_has_return_type() {
    let (syntax, resolver) = bind_types("fn foo() -> int { 42 } fn main() -> int { foo() }");

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let node = tree
        .iter()
        .find(|node| node.kind() == SyntaxKind::CallExpression)
        .unwrap();

    assert_eq!(
        *resolver.get_type_binding(&node.id).unwrap(),
        TypeId::INTEGER
    );
}
