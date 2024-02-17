use std::fmt::{self, Display, Formatter};

use lyneate::Report;
use serde::{Deserialize, Serialize};

use crate::{Identifier, SourcePosition, Type, TypeDefinition, Value};

use super::rw_lock_error::RwLockError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    /// Two value are incompatible for addition.
    CannotAdd {
        left: Value,
        right: Value,
        position: SourcePosition,
    },

    /// Two value are incompatible for subtraction.
    CannotSubtract {
        left: Value,
        right: Value,
        position: SourcePosition,
    },

    /// Two value are incompatible for multiplication.
    CannotMultiply {
        left: Value,
        right: Value,
        position: SourcePosition,
    },

    /// Two value are incompatible for dividing.
    CannotDivide {
        left: Value,
        right: Value,
        position: SourcePosition,
    },

    /// The attempted conversion is impossible.
    ConversionImpossible {
        initial_type: Type,
        target_type: Type,
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

    ExpectedMap {
        actual: Value,
    },

    ExpectedFunction {
        actual: Value,
    },

    /// A string, list, map or table value was expected.
    ExpectedCollection {
        actual: Value,
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
    ExpectedEnumDefintion {
        actual: TypeDefinition,
    },

    /// Failed to find a struct definition with this key.
    ExpectedStructDefintion {
        actual: TypeDefinition,
    },
}

impl ValidationError {
    pub fn create_report(&self, source: &str) -> String {
        let messages = match self {
            ValidationError::CannotAdd {
                left,
                right,
                position,
            } => vec![
                ((
                    position.start_byte..position.end_byte,
                    format!(""),
                    (255, 159, 64),
                )),
            ],
            ValidationError::CannotSubtract {
                left,
                right,
                position,
            } => todo!(),
            ValidationError::CannotMultiply {
                left,
                right,
                position,
            } => todo!(),
            ValidationError::CannotDivide {
                left,
                right,
                position,
            } => todo!(),
            ValidationError::ConversionImpossible {
                initial_type,
                target_type,
            } => todo!(),
            ValidationError::ExpectedString { actual } => todo!(),
            ValidationError::ExpectedInteger { actual } => todo!(),
            ValidationError::ExpectedFloat { actual } => todo!(),
            ValidationError::ExpectedNumber { actual } => todo!(),
            ValidationError::ExpectedNumberOrString { actual } => todo!(),
            ValidationError::ExpectedBoolean { actual } => todo!(),
            ValidationError::ExpectedList { actual } => todo!(),
            ValidationError::ExpectedMinLengthList {
                minimum_len,
                actual_len,
            } => todo!(),
            ValidationError::ExpectedFixedLenList {
                expected_len,
                actual,
            } => todo!(),
            ValidationError::ExpectedMap { actual } => todo!(),
            ValidationError::ExpectedFunction { actual } => todo!(),
            ValidationError::ExpectedCollection { actual } => todo!(),
            ValidationError::ExpectedBuiltInFunctionArgumentAmount {
                function_name,
                expected,
                actual,
            } => todo!(),
            ValidationError::ExpectedFunctionArgumentAmount {
                expected,
                actual,
                position,
            } => todo!(),
            ValidationError::ExpectedFunctionArgumentMinimum {
                minumum_expected,
                actual,
                position,
            } => todo!(),
            ValidationError::RwLock(_) => todo!(),
            ValidationError::TypeCheck {
                expected,
                actual,
                position,
            } => vec![(
                position.start_byte..position.end_byte,
                format!("Type {actual} is incompatible with {expected}."),
                (200, 200, 200),
            )],
            ValidationError::TypeCheckExpectedFunction { actual, position } => todo!(),
            ValidationError::VariableIdentifierNotFound(_) => todo!(),
            ValidationError::TypeDefinitionNotFound(_) => todo!(),
            ValidationError::ExpectedEnumDefintion { actual } => todo!(),
            ValidationError::ExpectedStructDefintion { actual } => todo!(),
        };

        Report::new_byte_spanned(source, messages).display_str()
    }

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
