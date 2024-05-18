use std::{io, sync::PoisonError};

use chumsky::{prelude::Rich, span::Span};

use crate::{
    abstract_tree::{SourcePosition, Type},
    identifier::Identifier,
    lexer::Token,
};

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse {
        expected: String,
        span: (usize, usize),
        found: Option<String>,
    },
    Lex {
        expected: String,
        span: (usize, usize),
        reason: String,
    },
    Runtime {
        error: RuntimeError,
        position: SourcePosition,
    },
    Validation {
        error: ValidationError,
        position: SourcePosition,
    },
}

impl From<Rich<'_, char>> for Error {
    fn from(error: Rich<'_, char>) -> Self {
        Error::Lex {
            expected: error.expected().map(|error| error.to_string()).collect(),
            span: (error.span().start(), error.span().end()),
            reason: error.reason().to_string(),
        }
    }
}

impl<'src> From<Rich<'_, Token<'src>>> for Error {
    fn from(error: Rich<'_, Token<'src>>) -> Self {
        Error::Parse {
            expected: error.expected().map(|error| error.to_string()).collect(),
            span: (error.span().start(), error.span().end()),
            found: error.found().map(|token| token.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    Io(io::Error),
    RwLockPoison(RwLockPoisonError),
    ValidationFailure(ValidationError),
}

impl From<RwLockPoisonError> for RuntimeError {
    fn from(error: RwLockPoisonError) -> Self {
        RuntimeError::RwLockPoison(error)
    }
}

impl<T> From<PoisonError<T>> for RuntimeError {
    fn from(_: PoisonError<T>) -> Self {
        RuntimeError::RwLockPoison(RwLockPoisonError)
    }
}

impl From<ValidationError> for RuntimeError {
    fn from(error: ValidationError) -> Self {
        RuntimeError::ValidationFailure(error)
    }
}

impl From<io::Error> for RuntimeError {
    fn from(error: io::Error) -> Self {
        RuntimeError::Io(error)
    }
}

impl PartialEq for RuntimeError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeError::Io(_), RuntimeError::Io(_)) => false,
            (RuntimeError::RwLockPoison(_), RuntimeError::RwLockPoison(_)) => true,
            (RuntimeError::ValidationFailure(left), RuntimeError::ValidationFailure(right)) => {
                left == right
            }
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    CannotIndex {
        r#type: Type,
        position: SourcePosition,
    },
    CannotIndexWith {
        collection_type: Type,
        collection_position: SourcePosition,
        index_type: Type,
        index_position: SourcePosition,
    },
    ExpectedBoolean {
        actual: Type,
        position: SourcePosition,
    },
    ExpectedFunction {
        actual: Type,
        position: SourcePosition,
    },
    ExpectedIntegerOrFloat(SourcePosition),
    ExpectedIntegerFloatOrString {
        actual: Type,
        position: SourcePosition,
    },
    ExpectedValue(SourcePosition),
    InterpreterExpectedReturn(SourcePosition),
    RwLockPoison(RwLockPoisonError),
    TypeCheck {
        /// The mismatch that caused the error.
        conflict: TypeConflict,

        /// The position of the item that gave the "actual" type.
        actual_position: SourcePosition,

        /// The position of the item that gave the "expected" type.
        expected_position: SourcePosition,
    },
    WrongArguments {
        expected: Vec<Type>,
        actual: Vec<Type>,
    },
    VariableNotFound {
        identifier: Identifier,
        position: SourcePosition,
    },
    PropertyNotFound {
        identifier: Identifier,
        position: SourcePosition,
    },
}

impl From<RwLockPoisonError> for ValidationError {
    fn from(error: RwLockPoisonError) -> Self {
        ValidationError::RwLockPoison(error)
    }
}

impl<T> From<PoisonError<T>> for ValidationError {
    fn from(_: PoisonError<T>) -> Self {
        ValidationError::RwLockPoison(RwLockPoisonError)
    }
}

#[derive(Debug, PartialEq)]
pub struct RwLockPoisonError;

impl<T> From<PoisonError<T>> for RwLockPoisonError {
    fn from(_: PoisonError<T>) -> Self {
        RwLockPoisonError
    }
}

#[derive(Debug, PartialEq)]
pub struct TypeConflict {
    pub actual: Type,
    pub expected: Type,
}
