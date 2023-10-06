//! Pattern matching.
//!
//! Note that this module is called "match" but is escaped as "r#match" because
//! "match" is a keyword in Rust.

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Match {}

impl AbstractTree for Match {
    fn from_syntax_node(_node: Node, _source: &str) -> Result<Self> {
        todo!()
    }

    fn run(&self, _context: &mut VariableMap) -> Result<Value> {
        todo!()
    }
}
