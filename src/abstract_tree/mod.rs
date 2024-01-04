//! Abstract, executable representations of corresponding items found in Dust
//! source code. The types that implement [AbstractTree] are inteded to be
//! created by an [Evaluator].
//!
//! When adding new lanugage features, first extend the grammar to recognize new
//! syntax nodes. Then add a new AbstractTree type using the existing types as
//! examples.

pub mod assignment;
pub mod block;
pub mod built_in_value;
pub mod expression;
pub mod r#for;
pub mod function_call;
pub mod function_expression;
pub mod function_node;
pub mod identifier;
pub mod if_else;
pub mod index;
pub mod index_assignment;
pub mod index_expression;
pub mod logic;
pub mod r#match;
pub mod math;
pub mod statement;
pub mod type_definition;
pub mod value_node;
pub mod r#while;
pub mod r#yield;

pub use {
    assignment::*, block::*, built_in_value::*, expression::*, function_call::*,
    function_expression::*, function_node::*, identifier::*, if_else::*, index::*,
    index_assignment::IndexAssignment, index_expression::*, logic::*, math::*, r#for::*,
    r#match::*, r#while::*, r#yield::*, statement::*, type_definition::*, value_node::*,
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

    fn check_type(&self, _context: &Map) -> Result<()> {
        for statement in &self.statements {
            if let Statement::Return(inner_statement) = statement {
                return inner_statement.check_type(_context);
            } else {
                statement.check_type(_context)?;
            }
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let mut value = Value::none();

        for statement in &self.statements {
            if let Statement::Return(inner_statement) = statement {
                return inner_statement.run(source, context);
            } else {
                value = statement.run(source, context)?;
            }
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

    /// Verify the type integrity of the node.
    fn check_type(&self, _context: &Map) -> Result<()> {
        Ok(())
    }

    /// Execute dust code by traversing the tree.
    fn run(&self, source: &str, context: &Map) -> Result<Value>;

    fn expected_type(&self, context: &Map) -> Result<Type>;
}
