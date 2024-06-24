use std::{io, sync::PoisonError as StdPoisonError};

use chumsky::{prelude::Rich, span::Span};

use crate::{
    abstract_tree::{r#type::Type, Expression, SourcePosition, TypeConstructor},
    identifier::Identifier,
    lexer::Token,
};

#[derive(Debug, PartialEq)]
pub enum DustError {
    Lex {
        expected: String,
        span: (usize, usize),
        reason: String,
    },
    Parse {
        expected: String,
        span: (usize, usize),
        found: Option<String>,
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

impl From<Rich<'_, char>> for DustError {
    fn from(error: Rich<'_, char>) -> Self {
        DustError::Lex {
            expected: error.expected().map(|error| error.to_string()).collect(),
            span: (error.span().start(), error.span().end()),
            reason: error.reason().to_string(),
        }
    }
}

impl<'src> From<Rich<'_, Token<'src>>> for DustError {
    fn from(error: Rich<'_, Token<'src>>) -> Self {
        DustError::Parse {
            expected: error.expected().map(|error| error.to_string()).collect(),
            span: (error.span().start(), error.span().end()),
            found: error.found().map(|token| token.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    Io(io::Error),
    RwLockPoison(PoisonError),
    ValidationFailure(ValidationError),
    SerdeJson(serde_json::Error),
}

impl From<PoisonError> for RuntimeError {
    fn from(error: PoisonError) -> Self {
        RuntimeError::RwLockPoison(error)
    }
}

impl<T> From<StdPoisonError<T>> for RuntimeError {
    fn from(_: StdPoisonError<T>) -> Self {
        RuntimeError::RwLockPoison(PoisonError)
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

impl From<serde_json::Error> for RuntimeError {
    fn from(error: serde_json::Error) -> Self {
        RuntimeError::SerdeJson(error)
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
    BuiltInFunctionFailure(&'static str),
    CannotAssignToNone(SourcePosition),
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
    ExpectedString {
        actual: Type,
        position: SourcePosition,
    },
    ExpectedList {
        actual: Type,
        position: SourcePosition,
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
    FullTypeNotKnown {
        identifier: Identifier,
        position: SourcePosition,
    },
    ExpectedExpression(SourcePosition),
    RwLockPoison(PoisonError),
    TypeCheck {
        /// The mismatch that caused the error.
        conflict: TypeConflict,

        /// The position of the item that gave the "actual" type.
        actual_position: SourcePosition,

        /// The position of the item that gave the "expected" type.
        expected_position: Option<SourcePosition>,
    },
    WrongTypeArguments {
        parameters: Vec<Identifier>,
        arguments: Vec<TypeConstructor>,
    },
    WrongValueArguments {
        parameters: Vec<(Identifier, Type)>,
        arguments: Vec<Expression>,
    },
    VariableNotFound {
        identifier: Identifier,
        position: SourcePosition,
    },
    FieldNotFound {
        identifier: Identifier,
        position: SourcePosition,
    },
    EnumDefinitionNotFound {
        identifier: Identifier,
        position: Option<SourcePosition>,
    },
    EnumVariantNotFound {
        identifier: Identifier,
        position: SourcePosition,
    },
}

impl From<PoisonError> for ValidationError {
    fn from(error: PoisonError) -> Self {
        ValidationError::RwLockPoison(error)
    }
}

impl<T> From<StdPoisonError<T>> for ValidationError {
    fn from(_: StdPoisonError<T>) -> Self {
        ValidationError::RwLockPoison(PoisonError)
    }
}

#[derive(Debug, PartialEq)]
pub struct PoisonError;

impl<T> From<StdPoisonError<T>> for PoisonError {
    fn from(_: StdPoisonError<T>) -> Self {
        PoisonError
    }
}

#[derive(Debug, PartialEq)]
pub struct TypeConflict {
    pub actual: Type,
    pub expected: Type,
}
