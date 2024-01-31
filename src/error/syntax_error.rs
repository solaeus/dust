use serde::{Deserialize, Serialize};
use tree_sitter::Point;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SyntaxError {
    UnexpectedSyntaxNode {
        expected: String,
        actual: String,

        #[serde(skip)]
        location: Point,

        relevant_source: String,
    },
}
