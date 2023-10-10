//! Abstract, executable representations of corresponding items found in Dust
//! source code. The types that implement [AbstractTree] are inteded to be
//! created by an [Evaluator].
//!
//! When adding new lanugage features, first extend the grammar to recognize new
//! syntax nodes. Then add a new AbstractTree type using the existing types as
//! examples.

pub mod assignment;
pub mod expression;
pub mod function_call;
pub mod identifier;
pub mod if_else;
pub mod item;
pub mod logic;
pub mod r#match;
pub mod math;
pub mod statement;
pub mod tool;
pub mod value_node;
pub mod r#while;

pub use {
    assignment::*, expression::*, function_call::*, identifier::*, if_else::*, item::*, logic::*,
    math::*, r#match::*, statement::*,
};

use tree_sitter::Node;

use crate::{Result, Value, VariableMap};

/// This trait is implemented by the Evaluator's internal types to form an
/// executable tree that resolves to a single value.
pub trait AbstractTree: Sized {
    /// Interpret the syntax tree at the given node and return the abstraction.
    ///
    /// This function is used to convert nodes in the Tree Sitter concrete
    /// syntax tree into executable nodes in an abstract tree. This function is
    /// where the tree should be traversed by accessing sibling and child nodes.
    /// Each node in the CST should be traversed only once.
    ///
    /// If necessary, the source code can be accessed directly by getting the
    /// node's byte range.
    fn from_syntax_node(source: &str, node: Node) -> Result<Self>;

    /// Execute dust code by traversing the tree
    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value>;
}
