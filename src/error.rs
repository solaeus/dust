//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.

use tree_sitter::Node;

use crate::{value::Value, Identifier};

use std::{fmt, io, num::ParseFloatError, string::FromUtf8Error, sync::PoisonError, time};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, PartialEq)]
pub enum Error {
    UnexpectedSyntaxNode {
        expected: &'static str,
        actual: &'static str,
        location: tree_sitter::Point,
        relevant_source: String,
    },

    /// The 'assert' macro did not resolve successfully.
    AssertEqualFailed {
        expected: Value,
        actual: Value,
    },

    /// The 'assert' macro did not resolve successfully.
    AssertFailed,

    /// A row was inserted to a table with the wrong amount of values.
    WrongColumnAmount {
        expected: usize,
        actual: usize,
    },

    /// An operator was called with the wrong amount of arguments.
    ExpectedOperatorArgumentAmount {
        expected: usize,
        actual: usize,
    },

    /// A function was called with the wrong amount of arguments.
    ExpectedToolArgumentAmount {
        tool_name: &'static str,
        expected: usize,
        actual: usize,
    },

    /// A function was called with the wrong amount of arguments.
    ExpectedAtLeastFunctionArgumentAmount {
        identifier: String,
        minimum: usize,
        actual: usize,
    },

    ExpectedString {
        actual: Value,
    },

    ExpectedInteger {
        actual: Value,
    },

    ExpectedFloat {
        actual: Value,
    },

    /// An integer, floating point or value was expected.
    ExpectedNumber {
        actual: Value,
    },

    /// An integer, floating point or string value was expected.
    ExpectedNumberOrString {
        actual: Value,
    },

    ExpectedBoolean {
        actual: Value,
    },

    ExpectedList {
        actual: Value,
    },

    ExpectedMinLengthList {
        minimum_len: usize,
        actual_len: usize,
    },

    ExpectedFixedLenList {
        expected_len: usize,
        actual: Value,
    },

    ExpectedEmpty {
        actual: Value,
    },

    ExpectedMap {
        actual: Value,
    },

    ExpectedTable {
        actual: Value,
    },

    ExpectedFunction {
        actual: Value,
    },

    /// A string, list, map or table value was expected.
    ExpectedCollection {
        actual: Value,
    },

    /// A `VariableIdentifier` operation did not find its value in the context.
    VariableIdentifierNotFound(String),

    /// A `FunctionIdentifier` operation did not find its value in the context.
    FunctionIdentifierNotFound(Identifier),

    /// The function failed due to an external error.
    ToolFailure(String),

    /// A custom error explained by its message.
    CustomMessage(String),
}

impl Error {
    pub fn expect_syntax_node(source: &str, expected: &'static str, actual: Node) -> Result<()> {
        if expected == actual.kind() {
            Ok(())
        } else {
            Err(Error::UnexpectedSyntaxNode {
                expected,
                actual: actual.kind(),
                location: actual.start_position(),
                relevant_source: source[actual.byte_range()].to_string(),
            })
        }
    }

    pub fn expect_tool_argument_amount(
        tool_name: &'static str,
        expected: usize,
        actual: usize,
    ) -> Result<()> {
        if expected == actual {
            Ok(())
        } else {
            Err(Error::ExpectedToolArgumentAmount {
                tool_name,
                expected,
                actual,
            })
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(value: time::SystemTimeError) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            AssertEqualFailed { expected, actual } => {
                write!(f, "Equality assertion failed")?;

                if expected.is_table() {
                    write!(f, "\n{expected}\n")?;
                } else {
                    write!(f, " {expected} ")?;
                }

                write!(f, "does not equal")?;

                if actual.is_table() {
                    write!(f, "\n{actual}")
                } else {
                    write!(f, " {actual}.")
                }
            }
            AssertFailed => write!(
                f,
                "Assertion failed. A false value was passed to \"assert\"."
            ),
            ExpectedOperatorArgumentAmount { expected, actual } => write!(
                f,
                "An operator expected {} arguments, but got {}.",
                expected, actual
            ),
            ExpectedToolArgumentAmount {
                tool_name,
                expected,
                actual,
            } => write!(
                f,
                "{tool_name} expected {expected} arguments, but got {actual}.",
            ),
            ExpectedAtLeastFunctionArgumentAmount {
                minimum,
                actual,
                identifier,
            } => write!(
                f,
                "{identifier} expected a minimum of {minimum} arguments, but got {actual}.",
            ),
            ExpectedString { actual } => {
                write!(f, "Expected a string but got {:?}.", actual)
            }
            ExpectedInteger { actual } => write!(f, "Expected an integer, but got {:?}.", actual),
            ExpectedFloat { actual } => write!(f, "Expected a float, but got {:?}.", actual),
            ExpectedNumber { actual } => {
                write!(f, "Expected a float or integer but got {:?}.", actual)
            }
            ExpectedNumberOrString { actual } => {
                write!(f, "Expected a number or string, but got {:?}.", actual)
            }
            ExpectedBoolean { actual } => {
                write!(f, "Expected a boolean, but got {:?}.", actual)
            }
            ExpectedList { actual } => write!(f, "Expected a list, but got {:?}.", actual),
            ExpectedMinLengthList {
                minimum_len,
                actual_len,
            } => write!(
                f,
                "Expected a list of at least {minimum_len} values, but got one with {actual_len}.",
            ),
            ExpectedFixedLenList {
                expected_len,
                actual,
            } => write!(
                f,
                "Expected a list of len {}, but got {:?}.",
                expected_len, actual
            ),
            ExpectedEmpty { actual } => write!(f, "Expected an empty value, but got {:?}.", actual),
            ExpectedMap { actual } => write!(f, "Expected a map, but got {:?}.", actual),
            ExpectedTable { actual } => write!(f, "Expected a table, but got {:?}.", actual),
            ExpectedFunction { actual } => {
                write!(f, "Expected function, but got {:?}.", actual)
            }
            ExpectedCollection { actual } => {
                write!(
                    f,
                    "Expected a string, list, map or table, but got {:?}.",
                    actual
                )
            }
            VariableIdentifierNotFound(identifier) => write!(
                f,
                "Variable identifier is not bound to anything by context: {}.",
                identifier
            ),
            FunctionIdentifierNotFound(identifier) => write!(
                f,
                "Function identifier is not bound to anything by context: {}.",
                identifier.inner()
            ),
            UnexpectedSyntaxNode {
                expected,
                actual,
                location,
                relevant_source,
            } => write!(
                f,
                "Expected {expected}, but got {actual} at {location}. Code: {relevant_source} ",
            ),
            WrongColumnAmount { expected, actual } => write!(
                f,
                "Wrong column amount. Expected {expected} but got {actual}."
            ),
            ToolFailure(message) => write!(f, "{message}"),
            CustomMessage(message) => write!(f, "{message}"),
        }
    }
}
