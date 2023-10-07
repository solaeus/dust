//! Top-level unit of Dust code.

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Result, Statement, Value, VariableMap};

/// An abstractiton of an independent unit of source code, or a comment.
///
/// Items are either comments, which do nothing, or statements, which can be run
/// to produce a single value or interact with a context by creating or
/// referencing variables.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Item {
    Comment(String),
    Statement(Statement),
}

impl AbstractTree for Item {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        debug_assert_eq!("item", node.kind());

        let child = node.child(0).unwrap();

        if child.kind() == "comment" {
            let byte_range = child.byte_range();
            let comment_text = &source[byte_range];

            Ok(Item::Comment(comment_text.to_string()))
        } else if child.kind() == "statement" {
            Ok(Item::Statement(Statement::from_syntax_node(child, source)?))
        } else {
            Err(Error::UnexpectedSyntax {
                expected: "comment or statement",
                actual: child.kind(),
                location: child.start_position(),
                relevant_source: source[node.byte_range()].to_string(),
            })
        }
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        match self {
            Item::Comment(text) => Ok(Value::String(text.clone())),
            Item::Statement(statement) => statement.run(context),
        }
    }
}
