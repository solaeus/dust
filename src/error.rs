use std::{io, ops::Range, sync::PoisonError};

use ariadne::{Color, Fmt, Label, ReportBuilder};
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
        let type_color = Color::Green;

        match self {
            Error::Parse { expected, span } => {
                let message = match expected.as_str() {
                    "" => "Invalid character.".to_string(),
                    expected => format!("Expected {expected}."),
                };

                builder = builder.with_note("Parsing error.");

                builder.add_label(Label::new(span.0..span.1).with_message(message));
            }
            Error::Lex { expected, span } => {
                let message = match expected.as_str() {
                    "" => "Invalid character.".to_string(),
                    expected => format!("Expected {expected}."),
                };

                builder = builder.with_note("Lexing error.");

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
                RuntimeError::Io(_) => todo!(),
            },
            Error::Validation { error, position } => match error {
                ValidationError::ExpectedBoolean { actual, position } => {
                    builder.add_label(Label::new(position.0..position.1).with_message(format!(
                        "Expected {} but got {}.",
                        "boolean".fg(type_color),
                        actual.fg(type_color)
                    )));
                }
                ValidationError::ExpectedIntegerOrFloat => {
                    builder.add_label(Label::new(position.0..position.1).with_message(format!(
                        "Expected {} or {}.",
                        "integer".fg(type_color),
                        "float".fg(type_color)
                    )));
                }
                ValidationError::RwLockPoison(_) => todo!(),
                ValidationError::TypeCheck {
                    conflict,
                    actual_position,
                    expected_position: expected_postion,
                } => {
                    let TypeConflict { actual, expected } = conflict;

                    builder.add_labels([
                        Label::new(expected_postion.0..expected_postion.1).with_message(format!(
                            "Type {} established here.",
                            expected.fg(type_color)
                        )),
                        Label::new(actual_position.0..actual_position.1)
                            .with_message(format!("Got type {} here.", actual.fg(type_color))),
                    ]);
                }
                ValidationError::VariableNotFound(identifier) => {
                    builder.add_label(
                        Label::new(position.0..position.1)
                            .with_message(format!("The variable {identifier} does not exist."))
                            .with_priority(1),
                    );
                }
                ValidationError::CannotIndex { r#type, position } => builder.add_label(
                    Label::new(position.0..position.1)
                        .with_message(format!("Cannot index into a {}.", r#type.fg(type_color))),
                ),
                ValidationError::CannotIndexWith {
                    collection_type,
                    index_type,
                    position,
                } => todo!(),
                ValidationError::InterpreterExpectedReturn => todo!(),
                ValidationError::ExpectedFunction { .. } => todo!(),
                ValidationError::ExpectedValue => todo!(),
                ValidationError::PropertyNotFound(_) => todo!(),
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
        index_type: Type,
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
    PropertyNotFound(Identifier),
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
