use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Identifier, SourcePosition, Type, Value};

use super::rw_lock_error::RwLockError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    /// The 'assert' macro did not resolve successfully.
    AssertEqualFailed {
        expected: Value,
        actual: Value,
        position: SourcePosition,
    },

    /// The 'assert' macro did not resolve successfully.
    AssertFailed { position: SourcePosition },

    /// The attempted conversion is impossible.
    ConversionImpossible {
        initial_type: Type,
        target_type: Type,
    },

    /// A built-in function was called with the wrong amount of arguments.
    ExpectedBuiltInFunctionArgumentAmount {
        function_name: String,
        expected: usize,
        actual: usize,
    },

    /// A function was called with the wrong amount of arguments.
    ExpectedFunctionArgumentAmount {
        expected: usize,
        actual: usize,
        position: SourcePosition,
    },

    /// A function was called with the wrong amount of arguments.
    ExpectedFunctionArgumentMinimum {
        minumum_expected: usize,
        actual: usize,
        position: SourcePosition,
    },

    /// Failed to read or write a map.
    ///
    /// See the [MapError] docs for more info.
    RwLock(RwLockError),

    TypeCheck {
        expected: Type,
        actual: Type,
        position: SourcePosition,
    },

    TypeCheckExpectedFunction {
        actual: Type,
        position: SourcePosition,
    },

    /// Failed to find a variable with a value for this key.
    VariableIdentifierNotFound(Identifier),
}

impl ValidationError {
    pub fn expect_argument_amount(
        function_name: &str,
        expected: usize,
        actual: usize,
    ) -> Result<(), Self> {
        if expected == actual {
            Ok(())
        } else {
            Err(ValidationError::ExpectedBuiltInFunctionArgumentAmount {
                function_name: function_name.to_string(),
                expected,
                actual,
            })
        }
    }
}

impl From<RwLockError> for ValidationError {
    fn from(_error: RwLockError) -> Self {
        ValidationError::RwLock(RwLockError)
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
