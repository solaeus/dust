#![allow(clippy::disallowed_macros)]
#![allow(clippy::disallowed_methods)]

mod const_item;
mod enum_item;
mod field_access_expression;
mod function_item;
mod impl_item;
mod impl_trait_item;
mod let_statement;
mod module_item;
mod path_expression;
mod struct_expression;
mod struct_item;
mod trait_item;
mod type_item;
mod type_notation;
mod use_item;

use std::path::{Path, PathBuf};

use crate::compiler::resolver::{
    Resolver,
    scopes::{ScopeId, ScopeKind},
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

fn find_function_body_scope(resolver: &Resolver, parent_scope_id: ScopeId) -> ScopeId {
    for (scope_id, scope) in resolver.scopes.iter() {
        if scope.kind != ScopeKind::Block {
            continue;
        }

        let mut ancestor_id = scope.parent;

        loop {
            if ancestor_id == Some(parent_scope_id) {
                return scope_id;
            }

            if ancestor_id.is_none() {
                break;
            }

            let ancestor = resolver.scopes.get_scope(ancestor_id.unwrap());

            if ancestor.kind == ScopeKind::Item || ancestor.kind == ScopeKind::Members {
                ancestor_id = ancestor.parent;
            } else {
                break;
            }
        }
    }

    panic!();
}
