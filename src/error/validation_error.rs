use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Identifier, SourcePosition, Type, TypeDefinition, Value};

use super::rw_lock_error::RwLockError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    /// Two value are incompatible for addition.
    CannotAdd { left: Value, right: Value },

    /// Two value are incompatible for subtraction.
    CannotSubtract { left: Value, right: Value },

    /// Two value are incompatible for multiplication.
    CannotMultiply { left: Value, right: Value },

    /// Two value are incompatible for dividing.
    CannotDivide { left: Value, right: Value },

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

    /// Failed to find a value with this key.
    VariableIdentifierNotFound(Identifier),

    /// Failed to find a type definition with this key.
    TypeDefinitionNotFound(Identifier),

    /// Failed to find an enum definition with this key.
    ExpectedEnumDefintion { actual: TypeDefinition },

    /// Failed to find a struct definition with this key.
    ExpectedStructDefintion { actual: TypeDefinition },
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
