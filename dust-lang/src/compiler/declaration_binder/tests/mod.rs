#![allow(clippy::disallowed_macros)]
#![allow(clippy::disallowed_methods)]

mod const_item;
mod enum_item;
mod expression_scoping;
mod field_access_expression;
mod function_item;
mod impl_item;
mod impl_trait_item;
mod let_statement;
mod module_item;
mod struct_item;
mod trait_item;
mod type_item;
mod type_notation;
mod use_item;

use std::path::{Path, PathBuf};

use smallvec::SmallVec;

use crate::{
    compiler::{
        declaration_binder::DeclarationBinder,
        resolver::{
            Resolver,
            scopes::{Scope, ScopeId, ScopeKind},
        },
        tests::bind_declarations,
    },
    error::ErrorKind,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Source},
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

fn cleanup_module_file(path: &Path) {
    let _ = std::fs::remove_file(path);

    if let Some(dir) = path.parent() {
        let _ = std::fs::remove_dir(dir);
    }
}

fn bind_declarations_with_errors(source: &Source) -> (Syntax, Resolver, ScopeId, Vec<ErrorKind>) {
    let mut syntax = Syntax::with_capacity(source.file_count());

    for (file_id, file) in source.iter() {
        let lexer = Lexer::with_validated_source(file.content_as_str());
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

    let main_root = syntax.get_tree(FileId::MAIN).unwrap().root().unwrap();

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
    let tree = syntax.get_tree(FileId::MAIN).unwrap();

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
