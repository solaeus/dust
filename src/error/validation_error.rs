use serde::{Deserialize, Serialize};

use crate::{Type, Value};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    /// The 'assert' macro did not resolve successfully.
    AssertEqualFailed {
        expected: Value,
        actual: Value,
    },

    /// The 'assert' macro did not resolve successfully.
    AssertFailed,

    TypeCheck {
        expected: Type,
        actual: Type,
    },

    TypeCheckExpectedFunction {
        actual: Type,
    },

    /// Failed to find a variable with a value for this key.
    VariableIdentifierNotFound(String),
}
