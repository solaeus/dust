use std::{ops::Range, sync::PoisonError};

use ariadne::{Label, ReportBuilder};
use chumsky::{prelude::Rich, span::Span};

use crate::{
    abstract_tree::{Identifier, SourcePosition, Type},
    lexer::Token,
};

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse {
        expected: String,
        span: (usize, usize),
    },
    Lex {
        expected: String,
        span: (usize, usize),
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

impl Error {
    pub fn build_report(
        self,
        mut builder: ReportBuilder<'_, Range<usize>>,
    ) -> ReportBuilder<'_, Range<usize>> {
        match self {
            Error::Parse { expected, span } => {
                let message = match expected.as_str() {
                    "" => "Invalid character.".to_string(),
                    expected => format!("Expected {expected}."),
                };

                builder.add_label(Label::new(span.0..span.1).with_message(message));
            }
            Error::Lex { expected, span } => {
                let message = match expected.as_str() {
                    "" => "Invalid character.".to_string(),
                    expected => format!("Expected {expected}."),
                };

                builder.add_label(Label::new(span.0..span.1).with_message(message));
            }
            Error::Runtime { error, position } => match error {
                RuntimeError::RwLockPoison(_) => todo!(),
                RuntimeError::ValidationFailure(validation_error) => {
                    builder =
                        Error::Validation {
                            error: validation_error,
                            position,
                        }
                        .build_report(builder.with_note(
                            "The interpreter failed to catch this error during validation.",
                        ));
                }
            },
            Error::Validation { error, position } => match error {
                ValidationError::ExpectedBoolean { actual, position } => {
                    builder.add_label(
                        Label::new(position.0..position.1)
                            .with_message(format!("Expected boolean but got {actual}.")),
                    );
                }
                ValidationError::ExpectedIntegerOrFloat => {
                    builder.add_label(
                        Label::new(position.0..position.1)
                            .with_message("Expected integer or float."),
                    );
                }
                ValidationError::RwLockPoison(_) => todo!(),
                ValidationError::TypeCheck {
                    conflict,
                    actual_position,
                    expected_position: expected_postion,
                } => {
                    let TypeConflict { actual, expected } = conflict;

                    builder.add_labels([
                        Label::new(expected_postion.0..expected_postion.1)
                            .with_message(format!("Type {expected} established here.")),
                        Label::new(actual_position.0..actual_position.1)
                            .with_message(format!("Got type {actual} here.")),
                    ]);
                }
                ValidationError::VariableNotFound(identifier) => {
                    builder.add_label(
                        Label::new(position.0..position.1)
                            .with_message(format!("The variable {identifier} does not exist."))
                            .with_priority(1),
                    );
                }
                ValidationError::CannotIndex(_) => todo!(),
                ValidationError::CannotIndexWith(_, _) => todo!(),
                ValidationError::InterpreterExpectedReturn => todo!(),
                ValidationError::ExpectedFunction { actual, position } => todo!(),
                ValidationError::ExpectedValue => todo!(),
            },
        }

        builder
    }
}

impl From<Rich<'_, char>> for Error {
    fn from(error: Rich<'_, char>) -> Self {
        Error::Lex {
            expected: error.expected().map(|error| error.to_string()).collect(),
            span: (error.span().start(), error.span().end()),
        }
    }
}

impl<'src> From<Rich<'_, Token<'src>>> for Error {
    fn from(error: Rich<'_, Token<'src>>) -> Self {
        Error::Parse {
            expected: error.expected().map(|error| error.to_string()).collect(),
            span: (error.span().start(), error.span().end()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    RwLockPoison(RwLockPoisonError),
    ValidationFailure(ValidationError),
}

impl From<RwLockPoisonError> for RuntimeError {
    fn from(error: RwLockPoisonError) -> Self {
        RuntimeError::RwLockPoison(error)
    }
}

impl From<ValidationError> for RuntimeError {
    fn from(error: ValidationError) -> Self {
        RuntimeError::ValidationFailure(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    CannotIndex(Type),
    CannotIndexWith(Type, Type),
    ExpectedBoolean {
        actual: Type,
        position: SourcePosition,
    },
    ExpectedFunction {
        actual: Type,
        position: SourcePosition,
    },
    ExpectedIntegerOrFloat,
    ExpectedValue,
    InterpreterExpectedReturn,
    RwLockPoison(RwLockPoisonError),
    TypeCheck {
        /// The mismatch that caused the error.
        conflict: TypeConflict,

        /// The position of the item that gave the "actual" type.
        actual_position: SourcePosition,

        /// The position of the item that gave the "expected" type.
        expected_position: SourcePosition,
    },
    VariableNotFound(Identifier),
}

impl From<RwLockPoisonError> for ValidationError {
    fn from(error: RwLockPoisonError) -> Self {
        ValidationError::RwLockPoison(error)
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
