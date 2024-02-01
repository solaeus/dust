//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.
mod runtime_error;
pub(crate) mod rw_lock_error;
mod syntax_error;
mod validation_error;

pub use runtime_error::RuntimeError;
pub use syntax_error::SyntaxError;
pub use validation_error::ValidationError;

use tree_sitter::{LanguageError, Node, Point};

use crate::{SourcePosition};

use std::fmt::{self, Formatter};

pub enum Error {
    Syntax(SyntaxError),

    Validation(ValidationError),

    Runtime(RuntimeError),

    ParserCancelled,

    Language(LanguageError),
}

impl Error {
    pub fn expect_syntax_node(
        source: &str,
        expected: &str,
        actual: Node,
    ) -> Result<(), SyntaxError> {
        log::info!("Converting {} to abstract node", actual.kind());

        if expected == actual.kind() {
            Ok(())
        } else if actual.is_error() {
            Err(SyntaxError::InvalidSource {
                source: source[actual.byte_range()].to_string(),
                position: SourcePosition::from(actual.range()),
            })
        } else {
            Err(SyntaxError::UnexpectedSyntaxNode {
                expected: expected.to_string(),
                actual: actual.kind().to_string(),
                location: actual.start_position(),
                relevant_source: source[actual.byte_range()].to_string(),
            })
        }
    }
}

impl From<SyntaxError> for Error {
    fn from(error: SyntaxError) -> Self {
        Error::Syntax(error)
    }
}

impl From<ValidationError> for Error {
    fn from(error: ValidationError) -> Self {
        Error::Validation(error)
    }
}

impl From<RuntimeError> for Error {
    fn from(error: RuntimeError) -> Self {
        Error::Runtime(error)
    }
}

impl From<LanguageError> for Error {
    fn from(error: LanguageError) -> Self {
        Error::Language(error)
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut Formatter) -> fmt::Result {
        use Error::*;

        match self {
            Syntax(_) => todo!(),
            Validation(_) => todo!(),
            Runtime(_) => todo!(),
            ParserCancelled => todo!(),
            Language(_) => todo!(),
        }
    }
}

fn get_position(position: &Point) -> String {
    format!("column {}, row {}", position.row + 1, position.column)
}
