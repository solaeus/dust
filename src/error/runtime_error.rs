use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Value;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum RuntimeError {
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

    /// Failed to find a variable with a value for this key.
    VariableIdentifierNotFound(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use RuntimeError::*;

        match self {
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
            ExpectedBuiltInFunctionArgumentAmount {
                function_name,
                expected,
                actual,
            } => todo!(),
            ExpectedFunctionArgumentAmount { expected, actual } => todo!(),
            ExpectedFunctionArgumentMinimum {
                source,
                minumum_expected,
                actual,
            } => todo!(),
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
            VariableIdentifierNotFound(_) => todo!(),
        }
    }
}
