#![allow(clippy::disallowed_macros)]
#![allow(clippy::disallowed_methods)]

use smallvec::SmallVec;

use crate::{
    compiler::{declaration_binder::DeclarationBinder, type_binder::TypeBinder},
    error::ErrorKind,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    resolver::{
        Resolver,
        declarations::{Definition, Visibility},
        scopes::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, components::FunctionItem, visitor::SyntaxVisitor},
};

pub fn bind_declarations(source: &Source) -> (Syntax, Resolver, ScopeId) {
    let mut syntax = Syntax::new(source.file_count());

    for (file_id, file) in source.iter() {
        let lexer = Lexer::from_utf8(file.content_as_str());
        let parser = Parser::new(file_id, lexer);
        let ParseResult {
            syntax_tree,
            errors,
            ..
        } = parser.parse();

        assert!(errors.is_empty(), "{errors:#?}");

        syntax.add_tree(syntax_tree);
    }

    let mut resolver = Resolver::new();
    let crate_scope_id = resolver.scopes.add_scope(Scope {
        kind: ScopeKind::Crate,
        parent: ScopeId::NONE,
        modules: SmallVec::new(),
        imports: SmallVec::new(),
    });

    let main_root = syntax.get_tree(SourceFileId::MAIN).unwrap().root().unwrap();

    let mut errors = Vec::new();
    let mut declaration_binder =
        DeclarationBinder::new(source, &syntax, &mut resolver, &mut errors, crate_scope_id);

    match declaration_binder.visit_root(main_root) {
        Ok(()) => {}
        Err(error) => errors.push(ErrorKind::Compile(error)),
    }

    assert!(errors.is_empty(), "{errors:#?}");

    (syntax, resolver, crate_scope_id)
}

pub fn type_bind_function(source_code: &str) -> (Syntax, Resolver, ScopeId) {
    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", source_code));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
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
        .get_tree(position.file_id)
        .and_then(|tree| tree.get_node(syntax_id))
        .unwrap();
    let FunctionItem { body, .. } = function_item.as_component().unwrap();

    resolver.type_parameter_map.clear();

    let type_parameter_declaration_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();

    for &type_parameter_declaration_id in type_parameter_declaration_ids {
        let inferred_type_id = resolver.types.create_inferred_type(None);
        resolver
            .type_parameter_map
            .insert(type_parameter_declaration_id, inferred_type_id);
    }

    let mut errors = Vec::new();
    let mut type_binder = TypeBinder::new(&mut resolver, &source);

    match type_binder.bind_function_body(body, return_type_id) {
        Ok(()) => {}
        Err(error) => errors.push(ErrorKind::Compile(error)),
    }

    assert!(errors.is_empty(), "{source_code}: {errors:#?}");

    (syntax, resolver, crate_scope_id)
}
