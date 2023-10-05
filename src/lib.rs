//! The Dust library is used to implement the Dust language, `src/main.rs` implements the command
//! line binary.
//!
//! Using this library is simple and straightforward, see the [inferface] module for instructions on
//! interpreting Dust code. Most of the language's features are implemented in the [tools] module.

pub use crate::{
    error::*,
    interface::*,
    value::{
        function::Function, table::Table, time::Time, value_type::ValueType,
        variable_map::VariableMap, Value,
    },
};

mod error;
mod evaluator;
mod interface;
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
pub const NODE_TYPES: &'static str = include_str!("../../../src/node-types.json");

/// Evaluate the given source code.
///
/// Returns a vector of results from evaluating the source code. Each comment
/// and statemtent will have its own result.
///
/// ```rust
/// # use dust_lib::*;
/// assert_eq!(eval("1 + 2 + 3"), vec![Ok(Value::from(6))]);
/// ```
pub fn eval(source: &str) -> Vec<Result<Value>> {
    let mut context = VariableMap::new();

    eval_with_context(source, &mut context)
}

/// Evaluate the given source code with the given context.
///
/// ```rust
/// # use dust_lib::*;
/// let mut context = VariableMap::new();
///
/// context.set_value("one".into(), 1.into());
/// context.set_value("two".into(), 2.into());
/// context.set_value("three".into(), 3.into());
///
/// let dust_code = "one + two + three";
///
/// assert_eq!(
///     eval_with_context(dust_code, &mut context),
///     vec![Ok(Value::Empty), Ok(Value::from(6))]
/// );
/// ```
pub fn eval_with_context(source: &str, context: &mut VariableMap) -> Vec<Result<Value>> {
    let mut parser = Parser::new();
    parser.set_language(language()).unwrap();

    Evaluator::new(parser, context, source).run()
}

// Uncomment these to include any queries that this grammar contains

// pub const HIGHLIGHTS_QUERY: &'static str = include_str!("../../queries/highlights.scm");
// pub const INJECTIONS_QUERY: &'static str = include_str!("../../queries/injections.scm");
// pub const LOCALS_QUERY: &'static str = include_str!("../../queries/locals.scm");
// pub const TAGS_QUERY: &'static str = include_str!("../../queries/tags.scm");

#[cfg(test)]
mod tests {
    #[test]
    fn load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(super::language())
            .expect("Error loading dust language");
    }
}
