#![allow(clippy::disallowed_methods)]

mod enum_item;
mod expression_scoping;
mod function_item;
mod let_statement;
mod module_item;
mod struct_item;
mod type_notation;
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
    syntax::{Syntax, node::SyntaxKind, visitor::SyntaxVisitor},
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

fn bind_declarations(source: &Source) -> (Syntax, Resolver, ScopeId) {
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

fn bind_declarations_with_errors(source: &Source) -> (Syntax, Resolver, ScopeId, Vec<ErrorKind>) {
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

    (syntax, resolver, crate_scope_id, errors)
}

fn find_function_body_scope(
    syntax: &Syntax,
    resolver: &Resolver,
    parent_scope_id: ScopeId,
) -> ScopeId {
    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();

    for reader in tree.iter() {
        if reader.node.kind == SyntaxKind::BlockExpression
            && let Ok(&scope_id) = resolver.get_scope_binding(&reader.id)
        {
            let scope = resolver.scopes.get_scope(scope_id).unwrap();

            if scope.kind == ScopeKind::Function && scope.parent == parent_scope_id {
                return scope_id;
            }
        }
    }

    panic!();
}
