#![allow(clippy::disallowed_methods)]

use crate::{
    compiler::declaration_binder::DeclarationBinder,
    error::ErrorKind,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    resolver::{
        Resolver,
        scopes::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFileId},
    syntax::{Syntax, visitor::SyntaxVisitor},
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
        modules: Vec::new(),
        imports: Vec::new(),
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
