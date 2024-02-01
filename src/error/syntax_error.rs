use serde::{Deserialize, Serialize};
use tree_sitter::Point;

use crate::SourcePosition;

use super::rw_lock_error::RwLockError;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SyntaxError {
    /// Invalid user input.
    InvalidSource {
        source: String,
        position: SourcePosition,
    },

    RwLock(RwLockError),

    UnexpectedSyntaxNode {
        expected: String,
        actual: String,

        #[serde(skip)]
        location: Point,

        relevant_source: String,
    },
}

impl From<RwLockError> for SyntaxError {
    fn from(error: RwLockError) -> Self {
        SyntaxError::RwLock(error)
    }
}
