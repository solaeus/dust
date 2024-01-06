//! The Dust library is used to implement the Dust language, `src/main.rs` implements the command
//! line binary.
//!
//! Using this library is simple and straightforward, see the [inferface] module for instructions on
//! interpreting Dust code. Most of the language's features are implemented in the [tools] module.
pub use crate::{
    abstract_tree::*,
    built_in_functions::BuiltInFunction,
    error::*,
    interpret::*,
    value::{function::Function, list::List, map::Map, structure::Structure, Value},
};

pub use tree_sitter::Node as SyntaxNode;

mod abstract_tree;
pub mod built_in_functions;
mod error;
mod interpret;
mod value;

use tree_sitter::Language;

extern "C" {
    fn tree_sitter_dust() -> Language;
}

/// Get the tree-sitter [Language][] for this grammar.
///
/// [Language]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Language.html
pub fn language() -> Language {
    unsafe { tree_sitter_dust() }
}

/// The content of the [`node-types.json`][] file for this grammar.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const NODE_TYPES: &str = include_str!("../tree-sitter-dust/src/node-types.json");

// Uncomment these to include any queries that this grammar contains

// pub const HIGHLIGHTS_QUERY: &'static str = include_str!("../../queries/highlights.scm");
// pub const INJECTIONS_QUERY: &'static str = include_str!("../../queries/injections.scm");
// pub const LOCALS_QUERY: &'static str = include_str!("../../queries/locals.scm");
// pub const TAGS_QUERY: &'static str = include_str!("../../queries/tags.scm");

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(super::language())
            .expect("Error loading dust language");
    }
}
