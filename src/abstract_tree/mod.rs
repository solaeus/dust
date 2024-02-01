//! Abstract, executable representations of corresponding items found in Dust
//! source code. The types that implement [AbstractTree] are inteded to be
//! created by an [Evaluator].
//!
//! When adding new lanugage features, first extend the grammar to recognize new
//! syntax nodes. Then add a new AbstractTree type using the existing types as
//! examples.

pub mod assignment;
pub mod assignment_operator;
pub mod block;
pub mod built_in_value;
pub mod command;
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
pub mod logic_operator;
pub mod r#match;
pub mod math;
pub mod math_operator;
pub mod new;
pub mod statement;
pub mod r#type;
pub mod type_specification;
pub mod value_node;
pub mod r#while;
pub mod r#yield;

pub use {
    assignment::*, assignment_operator::*, block::*, built_in_value::*, command::*, expression::*,
    function_call::*, function_expression::*, function_node::*, identifier::*, if_else::*,
    index::*, index_assignment::IndexAssignment, index_expression::*, logic::*, logic_operator::*,
    math::*, math_operator::*, new::*, r#for::*, r#match::*, r#type::*, r#while::*, r#yield::*,
    statement::*, type_specification::*, value_node::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    Error, Map, SyntaxNode, Value,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourcePosition {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_row: usize,
    pub start_column: usize,
    pub end_row: usize,
    pub end_column: usize,
}

impl From<tree_sitter::Range> for SourcePosition {
    fn from(range: tree_sitter::Range) -> Self {
        SourcePosition {
            start_byte: range.start_byte,
            end_byte: range.end_byte,
            start_row: range.start_point.row + 1,
            start_column: range.start_point.column,
            end_row: range.end_point.row + 1,
            end_column: range.end_point.column,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Root {
    statements: Vec<Statement>,
}

// TODO Change Root to use tree sitter's cursor to traverse the statements
// instead of indexes. This will be more performant when there are a lot of
// top-level statements in the tree.
impl AbstractTree for Root {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "root", node)?;

        let statement_count = node.child_count();
        let mut statements = Vec::with_capacity(statement_count);

        for index in 0..statement_count {
            let statement_node = node.child(index).unwrap();
            let statement = Statement::from_syntax(statement_node, source, context)?;

            statements.push(statement);
        }

        Ok(Root { statements })
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        for statement in &self.statements {
            if let Statement::Return(inner_statement) = statement {
                return inner_statement.check_type(_source, _context);
            } else {
                statement.check_type(_source, _context)?;
            }
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
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

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        self.statements.last().unwrap().expected_type(context)
    }
}

impl Format for Root {
    fn format(&self, output: &mut String, indent_level: u8) {
        for (index, statement) in self.statements.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }
            statement.format(output, indent_level);
            output.push('\n');
        }
    }
}

/// This trait is implemented by the Evaluator's internal types to form an
/// executable tree that resolves to a single value.
pub trait AbstractTree: Sized + Format {
    /// Interpret the syntax tree at the given node and return the abstraction. Returns a syntax
    /// error if the source is invalid.
    ///
    /// This function is used to convert nodes in the Tree Sitter concrete
    /// syntax tree into executable nodes in an abstract tree. This function is
    /// where the tree should be traversed by accessing sibling and child nodes.
    /// Each node in the CST should be traversed only once.
    ///
    /// If necessary, the source code can be accessed directly by getting the
    /// node's byte range.
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError>;

    /// Return the type of the value that this abstract node will create when run. Returns a
    /// validation error if the tree is invalid.
    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError>;

    /// Verify the type integrity of the node. Returns a validation error if the tree is invalid.
    fn check_type(&self, source: &str, context: &Map) -> Result<(), ValidationError>;

    /// Execute this node's logic and return a value. Returns a runtime error if the node cannot
    /// resolve to a value.
    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError>;
}

pub trait Format {
    fn format(&self, output: &mut String, indent_level: u8);

    fn indent(output: &mut String, indent_level: u8) {
        for _ in 0..indent_level {
            output.push_str("    ");
        }
    }
}
