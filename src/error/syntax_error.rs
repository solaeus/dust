use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::SourcePosition;

use super::rw_lock_error::RwLockError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SyntaxError {
    /// Invalid user input.
    InvalidSource {
        position: SourcePosition,
    },

    RwLock(RwLockError),

    UnexpectedSyntaxNode {
        expected: String,
        actual: String,
        position: SourcePosition,
    },
}

impl SyntaxError {
    pub fn expect_syntax_node(expected: &str, actual: SyntaxNode) -> Result<(), SyntaxError> {
        log::info!("Converting {} to abstract node", actual.kind());

        if expected == actual.kind() {
            Ok(())
        } else if actual.is_error() {
            Err(SyntaxError::InvalidSource {
                position: SourcePosition::from(actual.range()),
            })
        } else {
            Err(SyntaxError::UnexpectedSyntaxNode {
                expected: expected.to_string(),
                actual: actual.kind().to_string(),
                position: SourcePosition::from(actual.range()),
            })
        }
    }
}

impl From<RwLockError> for SyntaxError {
    fn from(error: RwLockError) -> Self {
        SyntaxError::RwLock(error)
    }
}

impl Display for SyntaxError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
