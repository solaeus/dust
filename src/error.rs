//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.

use tree_sitter::{Node, Point};

use crate::{value::Value, BuiltInFunction, Type};

use std::{
    fmt::{self, Formatter},
    io,
    num::ParseFloatError,
    string::FromUtf8Error,
    sync::PoisonError,
    time,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, PartialEq)]
pub enum Error {
    WithContext {
        error: Box<Error>,
        location: Point,
        source: String,
    },

    UnexpectedSyntaxNode {
        expected: &'static str,
        actual: &'static str,
        location: Point,
        relevant_source: String,
    },

    TypeCheck {
        expected: Type,
        actual: Type,
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
    ExpectedArgumentAmount {
        function_name: &'static str,
        expected: usize,
        actual: usize,
    },

    /// A function was called with the wrong amount of arguments.
    ExpectedArgumentMinimum {
        function_name: &'static str,
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
    FunctionIdentifierNotFound(String),

    /// The function failed due to an external error.
    External(String),

    /// A custom error explained by its message.
    CustomMessage(String),

    /// Invalid user input.
    Syntax {
        source: String,
        location: Point,
    },

    SerdeJson(String),
}

impl Error {
    pub fn at_node(self, node: Node, source: &str) -> Self {
        Error::WithContext {
            error: Box::new(self),
            location: node.start_position(),
            source: source[node.byte_range()].to_string(),
        }
    }

    pub fn expect_syntax_node(source: &str, expected: &'static str, actual: Node) -> Result<()> {
        if expected == actual.kind() {
            Ok(())
        } else if actual.is_error() {
            Err(Error::Syntax {
                source: source[actual.byte_range()].to_string(),
                location: actual.start_position(),
            })
        } else {
            Err(Error::UnexpectedSyntaxNode {
                expected,
                actual: actual.kind(),
                location: actual.start_position(),
                relevant_source: source[actual.byte_range()].to_string(),
            })
        }
    }

    pub fn expect_argument_amount<F: BuiltInFunction>(
        function: &F,
        expected: usize,
        actual: usize,
    ) -> Result<()> {
        if expected == actual {
            Ok(())
        } else {
            Err(Error::ExpectedArgumentAmount {
                function_name: function.name(),
                expected,
                actual,
            })
        }
    }

    pub fn expect_argument_minimum<F: BuiltInFunction>(
        function: &F,
        minimum: usize,
        actual: usize,
    ) -> Result<()> {
        if actual < minimum {
            Ok(())
        } else {
            Err(Error::ExpectedArgumentMinimum {
                function_name: function.name(),
                minimum,
                actual,
            })
        }
    }

    pub fn is_type_check_error(&self, other: &Error) -> bool {
        match self {
            Error::WithContext { error, .. } => {
                debug_assert_eq!(error.as_ref(), other);

                error.as_ref() == other
            }
            _ => {
                debug_assert_eq!(self, other);

                self == other
            }
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Error::External(value.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::External(value.to_string())
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::External(value.to_string())
    }
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error::External(value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::External(value.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::External(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value.to_string())
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(value: time::SystemTimeError) -> Self {
        Error::External(value.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::External(value.to_string())
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
            AssertEqualFailed { expected, actual } => {
                write!(
                    f,
                    "Equality assertion failed. {expected} does not equal {actual}."
                )
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
            ExpectedArgumentAmount {
                function_name: tool_name,
                expected,
                actual,
            } => write!(
                f,
                "{tool_name} expected {expected} arguments, but got {actual}.",
            ),
            ExpectedArgumentMinimum {
                function_name,
                minimum,
                actual,
            } => write!(
                f,
                "{function_name} expected a minimum of {minimum} arguments, but got {actual}.",
            ),
            ExpectedString { actual } => {
                write!(f, "Expected a string but got {actual}.")
            }
            ExpectedInteger { actual } => write!(f, "Expected an integer, but got {actual}."),
            ExpectedFloat { actual } => write!(f, "Expected a float, but got {actual}."),
            ExpectedNumber { actual } => {
                write!(f, "Expected a float or integer but got {actual}.",)
            }
            ExpectedNumberOrString { actual } => {
                write!(f, "Expected a number or string, but got {actual}.")
            }
            ExpectedBoolean { actual } => {
                write!(f, "Expected a boolean, but got {actual}.")
            }
            ExpectedList { actual } => write!(f, "Expected a list, but got {actual}."),
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
            ExpectedEmpty { actual } => write!(f, "Expected an empty value, but got {actual}."),
            ExpectedMap { actual } => write!(f, "Expected a map, but got {actual}."),
            ExpectedTable { actual } => write!(f, "Expected a table, but got {actual}."),
            ExpectedFunction { actual } => {
                write!(f, "Expected function, but got {actual}.")
            }
            ExpectedCollection { actual } => {
                write!(
                    f,
                    "Expected a string, list, map or table, but got {actual}.",
                )
            }
            VariableIdentifierNotFound(key) => write!(
                f,
                "Variable identifier is not bound to anything by context: {key}.",
            ),
            FunctionIdentifierNotFound(key) => write!(
                f,
                "Function identifier is not bound to anything by context: {key}."
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
            External(message) => write!(f, "External error: {message}"),
            CustomMessage(message) => write!(f, "{message}"),
            Syntax { source, location } => {
                write!(f, "Syntax error at {location}, this is not valid: {source}")
            }
            TypeCheck { expected, actual } => write!(
                f,
                "Type check error. Expected type {expected} but got type {actual}."
            ),
            WithContext {
                error,
                location,
                source,
            } => write!(f, "{error} Occured at {location}: \"{source}\""),
            SerdeJson(message) => write!(f, "JSON processing error: {message}"),
        }
    }
}
