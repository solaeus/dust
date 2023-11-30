//! Pattern matching.
//!
//! Note that this module is called "match" but is escaped as "r#match" because
//! "match" is a keyword in Rust.

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Map, Result, TypeDefinition, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Match {}

impl AbstractTree for Match {
    fn from_syntax_node(_source: &str, _node: Node) -> Result<Self> {
        todo!()
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        todo!()
    }

    fn expected_type(&self, _context: &Map) -> Result<TypeDefinition> {
        todo!()
    }
}
