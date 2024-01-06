//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.

use serde::{Deserialize, Serialize};
use tree_sitter::{LanguageError, Node, Point};

use crate::{value::Value, SyntaxPosition, Type};

use std::{
    fmt::{self, Formatter},
    io,
    num::ParseFloatError,
    string::FromUtf8Error,
    sync::PoisonError,
    time,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Error {
    AtSourcePosition {
        error: Box<Error>,
        source: String,
        start_row: usize,
        start_column: usize,
        end_row: usize,
        end_column: usize,
    },

    UnexpectedSyntaxNode {
        expected: String,
        actual: String,
        #[serde(skip)]
        location: Point,
        relevant_source: String,
    },

    TypeCheck {
        expected: Type,
        actual: Type,
    },

    TypeCheckExpectedFunction {
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
    ExpectedBuiltInFunctionArgumentAmount {
        function_name: String,
        expected: usize,
        actual: usize,
    },

    /// A function was called with the wrong amount of arguments.
    ExpectedFunctionArgumentAmount {
        expected: usize,
        actual: usize,
    },

    /// A function was called with the wrong amount of arguments.
    ExpectedFunctionArgumentMinimum {
        source: String,
        minumum_expected: usize,
        actual: usize,
    },

    ExpectedFunctionType {
        actual: Type,
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

    ExpectedNone {
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

    ExpectedOption {
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
        #[serde(skip)]
        location: Point,
    },

    SerdeJson(String),

    ParserCancelled,
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

    pub fn expect_syntax_node(source: &str, expected: &str, actual: Node) -> Result<()> {
        if expected == actual.kind() {
            Ok(())
        } else if actual.is_error() {
            Err(Error::Syntax {
                source: source[actual.byte_range()].to_string(),
                location: actual.start_position(),
            })
        } else {
            Err(Error::UnexpectedSyntaxNode {
                expected: expected.to_string(),
                actual: actual.kind().to_string(),
                location: actual.start_position(),
                relevant_source: source[actual.byte_range()].to_string(),
            })
        }
    }

    pub fn expect_argument_amount(
        function_name: &str,
        expected: usize,
        actual: usize,
    ) -> Result<()> {
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

    pub fn is_type_check_error(&self, other: &Error) -> bool {
        match self {
            Error::AtSourcePosition { error, .. } => error.as_ref() == other,
            _ => self == other,
        }
    }
}

impl From<LanguageError> for Error {
    fn from(value: LanguageError) -> Self {
        Error::External(value.to_string())
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
            ExpectedBuiltInFunctionArgumentAmount {
                function_name: tool_name,
                expected,
                actual,
            } => write!(
                f,
                "{tool_name} expected {expected} arguments, but got {actual}.",
            ),
            ExpectedFunctionArgumentAmount { expected, actual } => {
                write!(f, "Expected {expected} arguments, but got {actual}.",)
            }
            ExpectedFunctionArgumentMinimum {
                source,
                minumum_expected,
                actual,
            } => {
                write!(
                    f,
                    "{source} expected at least {minumum_expected} arguments, but got {actual}."
                )
            }
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
            ExpectedNone { actual } => write!(f, "Expected an empty value, but got {actual}."),
            ExpectedMap { actual } => write!(f, "Expected a map, but got {actual}."),
            ExpectedTable { actual } => write!(f, "Expected a table, but got {actual}."),
            ExpectedFunction { actual } => {
                write!(f, "Expected function, but got {actual}.")
            }
            ExpectedOption { actual } => write!(f, "Expected option, but got {actual}."),
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
            } => {
                let location = get_position(location);

                write!(
                    f,
                    "Expected {expected}, but got {actual} at {location}. Code: {relevant_source} ",
                )
            }
            WrongColumnAmount { expected, actual } => write!(
                f,
                "Wrong column amount. Expected {expected} but got {actual}."
            ),
            External(message) => write!(f, "External error: {message}"),
            CustomMessage(message) => write!(f, "{message}"),
            Syntax { source, location } => {
                let location = get_position(location);

                write!(f, "Syntax error at {location}: {source}")
            }
            TypeCheck { expected, actual } => write!(
                f,
                "Type check error. Expected type {expected} but got type {actual}."
            ),
            TypeCheckExpectedFunction { actual } => {
                write!(f, "Type check error. Expected a function but got {actual}.")
            }
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
            ExpectedFunctionType { actual } => write!(f, "Expected a function but got {actual}."),
        }
    }
}

fn get_position(position: &Point) -> String {
    format!("column {}, row {}", position.row + 1, position.column)
}
