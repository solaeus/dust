#![allow(clippy::disallowed_methods)]

use crate::{
    compiler::declaration_binder::DeclarationBinder,
    error::ErrorKind,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    resolver::{
        Resolver,
        declarations::{DeclarationId, Definition},
        scopes::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, visitor::SyntaxVisitor},
};

fn bind_declarations(source_code: &str) -> (Syntax, Resolver) {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("test", source_code));

    let lexer = Lexer::from_utf8(source_code);
    let parser = Parser::new(SourceFileId::MAIN, lexer);
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");

    let mut syntax = Syntax::new(source.file_count());

    syntax.add_tree(syntax_tree);

    let mut resolver = Resolver::new();
    let program_scope_id = resolver.scopes.add_scope(Scope {
        kind: ScopeKind::Crate,
        parent: ScopeId::NONE,
        modules: Vec::new(),
        imports: Vec::new(),
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
        Err(error) => errors.push(ErrorKind::Compile(error)),
    }

    assert!(errors.is_empty(), "{errors:#?}");

    (syntax, resolver)
}

fn find_declaration(resolver: &mut Resolver, name: &str) -> Option<(DeclarationId, Definition)> {
    let symbol_id = resolver.symbols.add_symbol(name);

    resolver
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.symbol_id == symbol_id)
        .map(|(id, declaration)| (id, declaration.definition))
}
