//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.
mod runtime_error;
mod syntax_error;
mod validation_error;

pub use runtime_error::RuntimeError;
pub use syntax_error::SyntaxError;
pub use validation_error::ValidationError;

use serde::{Deserialize, Serialize};
use tree_sitter::{LanguageError, Node, Point};

use crate::{value::Value, SyntaxPosition};

use std::{
    fmt::{self, Formatter},
    io,
    num::ParseFloatError,
    string::FromUtf8Error,
    sync::PoisonError,
    time,
};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Error {
    Parsing(SyntaxError),

    Verification(ValidationError),

    Runtime(RuntimeError),

    AtSourcePosition {
        error: Box<Error>,
        source: String,
        start_row: usize,
        start_column: usize,
        end_row: usize,
        end_column: usize,
    },

    /// The function failed due to an external error.
    External(String),

    /// A custom error explained by its message.
    CustomMessage(String),

    /// Invalid user input.
    Syntax {
        source: String,
        #[serde(skip)]
        location: Point,
    },

    SerdeJson(String),

    ParserCancelled,

    ParseFloat {
        reason: String,
    },

    ExpectedIterable {
        actual: Value,
    },
}

impl Error {
    pub fn at_source_position(self, source: &str, position: SyntaxPosition) -> Self {
        let byte_range = position.start_byte..position.end_byte;

        Error::AtSourcePosition {
            error: Box::new(self),
            source: source[byte_range].to_string(),
            start_row: position.start_row,
            start_column: position.start_column,
            end_row: position.end_row,
            end_column: position.end_column,
        }
    }

    pub fn expect_syntax_node(
        source: &str,
        expected: &str,
        actual: Node,
    ) -> Result<(), SyntaxError> {
        log::info!("Converting {} to abstract node", actual.kind());

        if expected == actual.kind() {
            Ok(())
        } else if actual.is_error() {
            Error::Syntax {
                source: source[actual.byte_range()].to_string(),
                location: actual.start_position(),
            }
        } else {
            SyntaxError::UnexpectedSyntaxNode {
                expected: expected.to_string(),
                actual: actual.kind().to_string(),
                location: actual.start_position(),
                relevant_source: source[actual.byte_range()].to_string(),
            }
        }
    }

    pub fn expect_argument_amount(
        function_name: &str,
        expected: usize,
        actual: usize,
    ) -> Result<(), ValidationError> {
        if expected == actual {
            Ok(())
        } else {
            Err(Error::ExpectedBuiltInFunctionArgumentAmount {
                function_name: function_name.to_string(),
                expected,
                actual,
            })
        }
    }

    pub fn is_error(&self, other: &Error) -> bool {
        match self {
            Error::AtSourcePosition { error, .. } => error.as_ref() == other,
            _ => self == other,
        }
    }
}

impl From<LanguageError> for Error {
    fn from(error: LanguageError) -> Self {
        Error::External(error.to_string())
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(error: PoisonError<T>) -> Self {
        Error::External(error.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::External(error.to_string())
    }
}

impl From<ParseFloatError> for Error {
    fn from(error: ParseFloatError) -> Self {
        Error::ParseFloat {
            reason: error.to_string(),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Self {
        Error::External(error.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::External(error.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::External(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(error.to_string())
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(error: time::SystemTimeError) -> Self {
        Error::External(error.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Error::External(error.to_string())
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Error::*;

        match self {
            AtSourcePosition {
                error,
                source,
                start_row,
                start_column,
                end_row,
                end_column,
            } => {
                write!(
                    f,
                    "{error} Occured at ({start_row}, {start_column}) to ({end_row}, {end_column}). Source: {source}"
                )
            }
            SerdeJson(message) => write!(f, "JSON processing error: {message}"),
            ParserCancelled => write!(
                f,
                "Parsing was cancelled either manually or because it took too long."
            ),
            ParseFloat { reason } => {
                write!(
                    f,
                    "Failed to parse a float value. Reason given: {}.",
                    reason
                )
            }
            ExpectedIterable { actual } => {
                write!(f, "Expected an iterable value but got {actual}.")
            }
            Parsing(_) => todo!(),
            Verification(_) => todo!(),
            Runtime(_) => todo!(),
            External(_) => todo!(),
            CustomMessage(_) => todo!(),
            Syntax { source, location } => todo!(),
        }
    }
}

fn get_position(position: &Point) -> String {
    format!("column {}, row {}", position.row + 1, position.column)
}
