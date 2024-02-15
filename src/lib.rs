//! The Dust library is used to parse, format and run dust source code.
//!
//! See the [interpret] module for more information.
//!
//! You can use this library externally by calling either of the "interpret"
//! functions or by constructing your own Interpreter.
pub use crate::{
    abstract_tree::*,
    built_in_functions::BuiltInFunction,
    context::{Context, ValueData},
    error::Error,
    interpret::*,
    value::*,
};

pub use tree_sitter::Node as SyntaxNode;

pub mod abstract_tree;
pub mod built_in_functions;
pub mod built_in_type_definitions;
pub mod built_in_types;
pub mod built_in_values;
pub mod context;
pub mod error;
pub mod interpret;
pub mod value;

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
