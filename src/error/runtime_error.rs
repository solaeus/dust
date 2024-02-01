use std::{
    fmt::{self, Debug, Display, Formatter},
    io,
    num::ParseFloatError,
    string::FromUtf8Error,
    time,
};

use crate::Value;

use super::rw_lock_error::RwLockError;

#[derive(PartialEq)]
pub enum RuntimeError {
    Csv(String),

    Io(String),

    Reqwest(String),

    Json(String),

    SystemTime(String),

    Toml(toml::de::Error),

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

    /// Failed to read or write a map.
    ///
    /// See the [MapError] docs for more info.
    RwLock(RwLockError),

    ParseFloat(ParseFloatError),

    Utf8(FromUtf8Error),

    /// Failed to find a variable with a value for this key.
    VariableIdentifierNotFound(String),

    /// A built-in function was called with the wrong amount of arguments.
    ExpectedBuiltInFunctionArgumentAmount {
        function_name: String,
        expected: usize,
        actual: usize,
    },
}

impl RuntimeError {
    pub fn expect_argument_amount(
        function_name: &str,
        expected: usize,
        actual: usize,
    ) -> Result<(), Self> {
        if expected == actual {
            Ok(())
        } else {
            Err(RuntimeError::ExpectedBuiltInFunctionArgumentAmount {
                function_name: function_name.to_string(),
                expected,
                actual,
            })
        }
    }
}

impl From<csv::Error> for RuntimeError {
    fn from(error: csv::Error) -> Self {
        RuntimeError::Csv(error.to_string())
    }
}

impl From<io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        RuntimeError::Io(error.to_string())
    }
}

impl From<reqwest::Error> for RuntimeError {
    fn from(error: reqwest::Error) -> Self {
        RuntimeError::Reqwest(error.to_string())
    }
}

impl From<serde_json::Error> for RuntimeError {
    fn from(error: serde_json::Error) -> Self {
        RuntimeError::Json(error.to_string())
    }
}

impl From<time::SystemTimeError> for RuntimeError {
    fn from(error: time::SystemTimeError) -> Self {
        RuntimeError::SystemTime(error.to_string())
    }
}

impl From<toml::de::Error> for RuntimeError {
    fn from(error: toml::de::Error) -> Self {
        RuntimeError::Toml(error)
    }
}

impl From<ParseFloatError> for RuntimeError {
    fn from(error: ParseFloatError) -> Self {
        RuntimeError::ParseFloat(error)
    }
}

impl From<FromUtf8Error> for RuntimeError {
    fn from(error: FromUtf8Error) -> Self {
        RuntimeError::Utf8(error)
    }
}

impl Display for RuntimeError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        use RuntimeError::*;

        match self {
            VariableIdentifierNotFound(_) => todo!(),
            RwLock(_) => todo!(),
            Csv(_) => todo!(),
            Io(_) => todo!(),
            Reqwest(_) => todo!(),
            Json(_) => todo!(),
            SystemTime(_) => todo!(),
            Toml(_) => todo!(),
            Utf8(_) => todo!(),
            ParseFloat(_) => todo!(),
            ExpectedBuiltInFunctionArgumentAmount {
                function_name: _,
                expected: _,
                actual: _,
            } => todo!(),
            ExpectedString { actual: _ } => todo!(),
            ExpectedInteger { actual: _ } => todo!(),
            ExpectedFloat { actual: _ } => todo!(),
            ExpectedNumber { actual: _ } => todo!(),
            ExpectedNumberOrString { actual: _ } => todo!(),
            ExpectedBoolean { actual: _ } => todo!(),
            ExpectedList { actual: _ } => todo!(),
            ExpectedMinLengthList {
                minimum_len: _,
                actual_len: _,
            } => todo!(),
            ExpectedFixedLenList {
                expected_len: _,
                actual: _,
            } => todo!(),
            ExpectedNone { actual: _ } => todo!(),
            ExpectedMap { actual: _ } => todo!(),
            ExpectedTable { actual: _ } => todo!(),
            ExpectedFunction { actual: _ } => todo!(),
            ExpectedOption { actual: _ } => todo!(),
            ExpectedCollection { actual: _ } => todo!(),
        }
    }
}

impl Debug for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl From<RwLockError> for RuntimeError {
    fn from(error: RwLockError) -> Self {
        RuntimeError::RwLock(error)
    }
}
