#![allow(clippy::disallowed_methods)]

mod arithmetic_expressions;
mod block_expression;
mod call_expression;
mod comparison_expressions;
mod enum_item;
mod function_item;
mod if_expression;
mod let_statement;
mod list_expression;
mod literal_expressions;
mod logical_expressions;
mod path_expression;
mod reassignment_statement;
mod struct_expression;
mod struct_item;
mod unary_expressions;
mod while_expression;

use smallvec::SmallVec;

use crate::{
    compiler::{declaration_binder::DeclarationBinder, type_binder::TypeBinder},
    error::ErrorKind,
    lexer::Lexer,
    parser::Parser,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind},
        scope_graph::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, visitor::SyntaxVisitor},
};

pub fn bind_types(source_code: &str) -> (Syntax, Resolver) {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("test", source_code));

    let lexer = Lexer::from_utf8(source_code);
    let parser = Parser::new(SourceFileId::MAIN, lexer);
    let parse_result = parser.parse();

    let mut syntax = Syntax::new(source.file_count());

    syntax.add_tree(parse_result.syntax_tree);

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
            Err(error) => errors.push(ErrorKind::Compile(error)),
        }
    }

    assert!(errors.is_empty(), "declaration binding errors: {errors:#?}");

    {
        let main_root = syntax.get_tree(SourceFileId::MAIN).unwrap().root().unwrap();
        let mut type_binder = TypeBinder::new(&syntax, &mut resolver, &mut errors);

        match type_binder.visit_root(main_root) {
            Ok(()) => {}
            Err(error) => errors.push(ErrorKind::Compile(error)),
        }
    }

    assert!(errors.is_empty(), "type binding errors: {errors:#?}");

    (syntax, resolver)
}

pub fn find_declaration(
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
