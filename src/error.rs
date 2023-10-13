//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.

use crate::{value::Value, Identifier};

use std::{fmt, io, time::SystemTimeError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, PartialEq)]
pub enum Error {
    UnexpectedSyntaxNode {
        expected: &'static str,
        actual: &'static str,
        location: tree_sitter::Point,
        relevant_source: String,
    },

    ExpectedChildNode {
        empty_node_sexp: String,
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

    ExpectedInt {
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

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<json::Error> for Error {
    fn from(value: json::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<git2::Error> for Error {
    fn from(value: git2::Error) -> Self {
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

impl From<SystemTimeError> for Error {
    fn from(value: SystemTimeError) -> Self {
        Error::ToolFailure(value.to_string())
    }
}

impl From<trash::Error> for Error {
    fn from(value: trash::Error) -> Self {
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
            AssertEqualFailed { expected, actual } => write!(
                f,
                "Equality assertion failed. {expected} does not equal {actual}."
            ),
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
                write!(f, "Expected a Value::String, but got {:?}.", actual)
            }
            ExpectedInt { actual } => write!(f, "Expected a Value::Int, but got {:?}.", actual),
            ExpectedFloat { actual } => write!(f, "Expected a Value::Float, but got {:?}.", actual),
            ExpectedNumber { actual } => write!(
                f,
                "Expected a Value::Float or Value::Int, but got {:?}.",
                actual
            ),
            ExpectedNumberOrString { actual } => write!(
                f,
                "Expected a Value::Number or a Value::String, but got {:?}.",
                actual
            ),
            ExpectedBoolean { actual } => {
                write!(f, "Expected a Value::Boolean, but got {:?}.", actual)
            }
            ExpectedList { actual } => write!(f, "Expected a Value::Tuple, but got {:?}.", actual),
            ExpectedFixedLenList {
                expected_len,
                actual,
            } => write!(
                f,
                "Expected a Value::Tuple of len {}, but got {:?}.",
                expected_len, actual
            ),
            ExpectedEmpty { actual } => write!(f, "Expected a Value::Empty, but got {:?}.", actual),
            ExpectedMap { actual } => write!(f, "Expected a Value::Map, but got {:?}.", actual),
            ExpectedTable { actual } => write!(f, "Expected a Value::Table, but got {:?}.", actual),
            ExpectedFunction { actual } => {
                write!(f, "Expected Value::Function, but got {:?}.", actual)
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
                "Expected syntax node {expected}, but got {actual} at {location}. Code: {relevant_source} ",
            ),
            ExpectedChildNode { empty_node_sexp } => todo!(),
            WrongColumnAmount { expected, actual } => todo!(),
            ToolFailure(_) => todo!(),
            CustomMessage(_) => todo!(),
        }
    }
}
