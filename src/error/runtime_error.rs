use std::{
    fmt::{self, Debug, Display, Formatter},
    io,
    num::ParseFloatError,
    string::FromUtf8Error,
    sync::PoisonError,
    time,
};

use crate::{Identifier, Type, Value};

use super::{rw_lock_error::RwLockError, ValidationError};

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    /// The attempted conversion is impossible.
    ConversionImpossible {
        value: Value,
        target_type: Type,
    },

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
    VariableIdentifierNotFound(Identifier),

    /// A built-in function was called with the wrong amount of arguments.
    ExpectedBuiltInFunctionArgumentAmount {
        function_name: String,
        expected: usize,
        actual: usize,
    },

    ValidationFailure(ValidationError),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<RwLockError> for RuntimeError {
    fn from(error: RwLockError) -> Self {
        RuntimeError::RwLock(error)
    }
}

impl<T> From<PoisonError<T>> for RuntimeError {
    fn from(_: PoisonError<T>) -> Self {
        RuntimeError::RwLock(RwLockError)
    }
}
