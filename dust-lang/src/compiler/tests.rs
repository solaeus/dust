#![allow(clippy::disallowed_macros)]
#![allow(clippy::disallowed_methods)]

use smallvec::smallvec;

use crate::{
    compiler::resolver::{
        Resolver,
        declarations::Definition,
        scopes::{ScopeId, ScopeKind},
    },
    compiler::{Compiler, declaration_binder::DeclarationBinder, type_binder::TypeBinder},
    error::ErrorKind,
    instruction::{Instruction, MemoryKind, OperandType},
    lexer::Lexer,
    parser::{ParseResult, Parser},
    program::Program,
    prototype::Prototype,
    source::{Source, SourceCode, SourceCodeId},
    syntax::{Syntax, components::FnItem},
};

fn compile(source_code: &str) -> Program {
    let mut source = Source::new();
    source.add_code(SourceCode::validated_borrowed("test", source_code));

    Compiler::new(source).compile(None).unwrap()
}

pub fn bind_declarations(source: &Source) -> (Syntax, Resolver, ScopeId) {
    let mut syntax = Syntax::with_capacity(source.file_count());

    for (source_id, file) in source.iter() {
        let lexer = Lexer::with_validated_source(file.content_as_str());
        let parser = Parser::new(source_id, lexer);
        let ParseResult {
            syntax_tree,
            errors,
            ..
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");

        syntax.add_tree(syntax_tree);
    }

    let mut resolver = Resolver::new();
    let crate_scope_id = resolver
        .scopes
        .enter_scope(ScopeKind::Module, ScopeId::NONE);

    let main_root = syntax.get_tree(SourceCodeId::MAIN).unwrap().root().unwrap();

    let mut errors = Vec::new();
    let mut declaration_binder =
        DeclarationBinder::new(source, &syntax, &mut resolver, &mut errors, crate_scope_id);

    match declaration_binder.bind_root(main_root) {
        Ok(()) => {}
        Err(error) => errors.push(ErrorKind::Compile(error)),
    }

    assert!(errors.is_empty(), "{errors:#?}");

    (syntax, resolver, crate_scope_id)
}

pub fn type_bind_function(source_code: &str) -> (Syntax, Resolver, ScopeId) {
    let mut source = Source::new();
    source.add_code(SourceCode::validated_borrowed("test", source_code));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id)
        .unwrap();
    let foo_declaration = *foo_declaration;

    let Definition::Function {
        type_parameters,
        return_type_id,
        ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let (position, syntax_id) = foo_declaration.syntax.unwrap();

    let function_item = syntax
        .get_tree(position.source_id)
        .and_then(|tree| tree.read_node(syntax_id))
        .unwrap();
    let FnItem { body, .. } = function_item.as_component().unwrap();

    resolver.type_parameter_map.clear();

    if type_parameters != ScopeId::NONE {
        let type_parameter_entries = resolver.scopes.get_namespace_entries(type_parameters);

        let type_parameter_ids: Vec<_> = type_parameter_entries
            .iter()
            .map(|&(_, declaration_id)| declaration_id)
            .collect();

        for type_parameter_declaration_id in type_parameter_ids {
            let inferred_type_id = resolver.types.create_inferred_type(None);
            resolver
                .type_parameter_map
                .insert(type_parameter_declaration_id, inferred_type_id);
        }
    }

    let mut errors = Vec::new();
    let mut type_binder = TypeBinder::new(&mut resolver, &source);

    match type_binder.bind_function_body(body.unwrap(), return_type_id) {
        Ok(()) => {}
        Err(error) => errors.push(ErrorKind::Compile(error)),
    }

    assert!(errors.is_empty(), "{source_code}: {errors:#?}");

    (syntax, resolver, crate_scope_id)
}

#[test]
fn function_call() {
    let program = compile(
        r#"
        fn add(a: i32, b: i32) -> i32 { a + b }

        fn main() -> i32 {
            add(1, 2)
        }
    "#,
    );

    assert_eq!(program.prototypes.len(), 2);

    let main = &program.prototypes[0];

    assert_eq!(
        main,
        &Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::call(0, MemoryKind::ENCODED, 1, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 0,
        }
    );
}

#[test]
fn struct_method_call() {
    let program = compile(
        r#"
        struct Foo {}

        impl Foo {
            fn value() -> i32 { 42 }
        }

        fn main() -> i32 {
            Foo::value()
        }
    "#,
    );

    assert_eq!(program.prototypes.len(), 2);

    assert_eq!(
        program.prototypes[0],
        Prototype {
            instructions: vec![
                Instruction::call(0, MemoryKind::ENCODED, 1, u16::MAX),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );

    assert_eq!(
        program.prototypes[1],
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn trait_method_call() {
    let program = compile(
        r#"
        struct Point {
            x: i32,
        }

        trait GetX {
            fn get_x(self) -> i32;
        }

        impl GetX for Point {
            fn get_x(self) -> i32 { self.x }
        }

        fn main() -> i32 {
            let point: Point = Point { x: 1 };
            point.get_x()
        }
    "#,
    );

    assert_eq!(program.prototypes.len(), 2);
}

#[test]
fn default_trait_method() {
    let program = compile(
        r#"
        struct Point {
            x: i32,
        }

        trait Answer {
            fn answer(self) -> i32 { 42 }
        }

        impl Answer for Point {}

        fn main() -> i32 {
            let point: Point = Point { x: 0 };
            point.answer()
        }
    "#,
    );

    assert_eq!(program.prototypes.len(), 2);
}

#[test]
fn default_method_calling_trait_method() {
    let program = compile(
        r#"
        struct Point {
            x: i32,
        }

        trait GetX {
            fn access_x(self) -> i32;
            fn get_x(self) -> i32 { self.access_x() }
        }

        impl GetX for Point {
            fn access_x(self) -> i32 { self.x }
        }

        fn main() -> i32 {
            let p: Point = Point { x: 5 };
            p.get_x()
        }
    "#,
    );

    assert_eq!(program.prototypes.len(), 3);
}

#[test]
fn generic_monomorphization() {
    let program = compile(
        r#"
        fn identity<T>(x: T) -> T { x }

        fn main() -> i32 {
            identity::<i32>(5)
        }
    "#,
    );

    assert_eq!(program.prototypes.len(), 2);

    assert_eq!(
        program.prototypes[0],
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 5),
                Instruction::call(0, MemoryKind::ENCODED, 1, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );

    assert_eq!(
        program.prototypes[1],
        Prototype {
            instructions: vec![Instruction::r#return(),],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 1,
        }
    );
}

#[test]
fn self_return_type() {
    let program = compile(
        r#"
        struct Point {
            x: i32,
        }

        trait Identity {
            fn identity(self) -> Self;
        }

        impl Identity for Point {
            fn identity(self) -> Self { self }
        }

        fn main() -> i32 {
            let p: Point = Point { x: 1 };
            p.identity().x
        }
    "#,
    );

    assert_eq!(program.prototypes.len(), 2);
}
