//! Abstract, executable representations of corresponding items found in Dust
//! source code. The types that implement [AbstractTree] are inteded to be
//! created by an [Interpreter].
pub mod r#as;
pub mod assignment;
pub mod assignment_operator;
pub mod block;
pub mod command;
pub mod enum_defintion;
pub mod enum_pattern;
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
pub mod map_node;
pub mod r#match;
pub mod match_pattern;
pub mod math;
pub mod math_operator;
pub mod statement;
pub mod struct_definition;
pub mod r#type;
pub mod type_definition;
pub mod type_specification;
pub mod value_node;
pub mod r#while;

pub use {
    assignment::*, assignment_operator::*, block::*, command::*, enum_defintion::*,
    enum_pattern::*, expression::*, function_call::*, function_expression::*, function_node::*,
    identifier::*, if_else::*, index::*, index_assignment::IndexAssignment, index_expression::*,
    logic::*, logic_operator::*, map_node::*, match_pattern::*, math::*, math_operator::*, r#as::*,
    r#for::*, r#match::*, r#type::*, r#while::*, statement::*, struct_definition::*,
    type_definition::*, type_specification::*, value_node::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, SyntaxError, ValidationError},
    SyntaxNode, Value,
};

/// A detailed report of a position in the source code string.
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

/// Abstraction that represents a whole, executable unit of dust code.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Root {
    statements: Vec<Statement>,
}

// TODO Change Root to use tree sitter's cursor to traverse the statements
// instead of indexes. This will be more performant when there are a lot of
// top-level statements in the tree.
impl AbstractTree for Root {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "root", node)?;

        let statement_count = node.child_count();
        let mut statements = Vec::with_capacity(statement_count);

        for index in 0..statement_count {
            let statement_node = node.child(index).unwrap();
            let statement = Statement::from_syntax(statement_node, source, context)?;

            statements.push(statement);
        }

        Ok(Root { statements })
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_source, _context)?;
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let mut value = Value::none();

        for statement in &self.statements {
            value = statement.run(source, context)?;

            if statement.is_return() {
                return Ok(value);
            }
        }

        Ok(value)
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
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
    /// Interpret the syntax tree at the given node and return the abstraction.
    /// Returns a syntax error if the source is invalid.
    ///
    /// This function is used to convert nodes in the Tree Sitter concrete
    /// syntax tree into executable nodes in an abstract tree. This function is
    /// where the tree should be traversed by accessing sibling and child nodes.
    /// Each node in the CST should be traversed only once.
    ///
    /// If necessary, the source code can be accessed directly by getting the
    /// node's byte range.
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError>;

    /// Return the type of the value that this abstract node will create when
    /// run. Returns a validation error if the tree is invalid.
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError>;

    /// Verify the type integrity of the node. Returns a validation error if the
    /// tree is invalid.
    fn validate(&self, source: &str, context: &Context) -> Result<(), ValidationError>;

    /// Execute this node's logic and return a value. Returns a runtime error if
    /// the node cannot resolve to a value.
    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError>;
}

pub trait Format {
    fn format(&self, output: &mut String, indent_level: u8);

    fn indent(output: &mut String, indent_level: u8) {
        for _ in 0..indent_level {
            output.push_str("    ");
        }
    }
}
