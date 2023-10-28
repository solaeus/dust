//! Abstract, executable representations of corresponding items found in Dust
//! source code. The types that implement [AbstractTree] are inteded to be
//! created by an [Evaluator].
//!
//! When adding new lanugage features, first extend the grammar to recognize new
//! syntax nodes. Then add a new AbstractTree type using the existing types as
//! examples.

pub mod assignment;
pub mod r#async;
pub mod expression;
pub mod filter;
pub mod find;
pub mod r#for;
pub mod function_call;
pub mod identifier;
pub mod if_else;
pub mod insert;
pub mod item;
pub mod logic;
pub mod r#match;
pub mod math;
pub mod remove;
pub mod select;
pub mod statement;
pub mod sublist;
pub mod tool;
pub mod transform;
pub mod value_node;
pub mod r#while;

pub use {
    assignment::*, expression::*, filter::*, find::*, function_call::*, identifier::*, if_else::*,
    insert::*, item::*, logic::*, math::*, r#async::*, r#for::*, r#match::*, r#while::*, remove::*,
    select::*, statement::*, sublist::*, tool::*, transform::*, value_node::*,
};

use tree_sitter::Node;

use crate::{Map, Result, Value};

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
    fn run(&self, source: &str, context: &mut Map) -> Result<Value>;
}
