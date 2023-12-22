//! Abstract, executable representations of corresponding items found in Dust
//! source code. The types that implement [AbstractTree] are inteded to be
//! created by an [Evaluator].
//!
//! When adding new lanugage features, first extend the grammar to recognize new
//! syntax nodes. Then add a new AbstractTree type using the existing types as
//! examples.

pub mod assignment;
pub mod block;
pub mod expression;
pub mod r#for;
pub mod function_call;
pub mod identifier;
pub mod if_else;
pub mod index;
pub mod index_assignment;
pub mod logic;
pub mod r#match;
pub mod math;
pub mod statement;
pub mod type_definition;
pub mod r#use;
pub mod value_node;
pub mod r#while;
pub mod r#yield;

pub use {
    assignment::*, block::*, expression::*, function_call::*, identifier::*, if_else::*, index::*,
    index_assignment::IndexAssignment, logic::*, math::*, r#for::*, r#match::*, r#use::*,
    r#while::*, r#yield::*, statement::*, type_definition::*, value_node::*,
};

use tree_sitter::Node;

use crate::{Error, Map, Result, Value};

pub struct Root {
    statements: Vec<Statement>,
}

impl AbstractTree for Root {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "root", node)?;

        let statement_count = node.child_count();
        let mut statements = Vec::with_capacity(statement_count);

        for index in 0..statement_count {
            let statement_node = node.child(index).unwrap();
            let statement = Statement::from_syntax_node(source, statement_node, context)?;

            statements.push(statement);
        }

        Ok(Root { statements })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let mut value = Value::Option(None);

        for statement in &self.statements {
            value = statement.run(source, context)?;
        }

        Ok(value)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.statements.last().unwrap().expected_type(context)
    }
}

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
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self>;

    /// Execute dust code by traversing the tree.
    fn run(&self, source: &str, context: &Map) -> Result<Value>;

    fn expected_type(&self, context: &Map) -> Result<Type>;
}
