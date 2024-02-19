use std::fmt::{self, Display, Formatter};

use colored::Colorize;
use lyneate::Report;
use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::SourcePosition;

use super::rw_lock_error::RwLockError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SyntaxError {
    /// Invalid user input.
    InvalidSource {
        expected: String,
        actual: String,
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
    pub fn create_report(&self, source: &str) -> String {
        let messages = match self {
            SyntaxError::InvalidSource { position, .. } => self
                .to_string()
                .split_inclusive(".")
                .map(|message_part| {
                    (
                        position.start_byte..position.end_byte,
                        message_part.to_string(),
                        (255, 200, 100),
                    )
                })
                .collect(),
            SyntaxError::RwLock(_) => todo!(),
            SyntaxError::UnexpectedSyntaxNode { position, .. } => {
                vec![(
                    position.start_byte..position.end_byte,
                    self.to_string(),
                    (255, 200, 100),
                )]
            }
        };

        Report::new_byte_spanned(source, messages).display_str()
    }

    pub fn expect_syntax_node(expected: &str, actual: SyntaxNode) -> Result<(), SyntaxError> {
        log::trace!("Converting {} to abstract node", actual.kind());

        if expected == actual.kind() {
            Ok(())
        } else if actual.is_error() {
            Err(SyntaxError::InvalidSource {
                expected: expected.to_owned(),
                actual: actual.kind().to_string(),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::InvalidSource {
                expected,
                actual,
                position,
            } => {
                let actual = if actual == "ERROR" {
                    "unrecognized characters"
                } else {
                    actual
                };

                write!(
                    f,
                    "Invalid syntax from ({}, {}) to ({}, {}). Exected {} but found {}.",
                    position.start_row,
                    position.start_column,
                    position.end_row,
                    position.end_column,
                    expected.bold().green(),
                    actual.bold().red(),
                )
            }
            SyntaxError::RwLock(_) => todo!(),
            SyntaxError::UnexpectedSyntaxNode {
                expected,
                actual,
                position,
            } => {
                write!(
                        f,
                        "Interpreter Error. Tried to parse {actual} as {expected} from ({}, {}) to ({}, {}).",
                        position.start_row,
                        position.start_column,
                        position.end_row,
                        position.end_column,
                    )
            }
        }
    }
}
