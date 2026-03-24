#![allow(clippy::disallowed_methods)]

mod enum_item;
mod function_item;
mod let_statement;
mod module_item;
mod struct_item;
mod use_item;

use std::path::PathBuf;

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

fn create_module_file(name: &str, content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "delcartion_binder_test_{:?}",
        std::thread::current().id()
    ));
    let path = dir.join(format!("{name}.rs"));

    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, content).unwrap();

    path
}

fn cleanup_module_file(path: &PathBuf) {
    let _ = std::fs::remove_file(path);

    if let Some(dir) = path.parent() {
        let _ = std::fs::remove_dir(dir);
    }
}

fn bind_declarations(source: &Source) -> (Resolver, ScopeId) {
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

    (resolver, crate_scope_id)
}
