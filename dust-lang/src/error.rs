use std::{io, sync::PoisonError};

use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};
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
        reason: String,
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

impl Error {
    pub fn build_report(self, source: &str) -> Result<Vec<u8>, io::Error> {
        let (mut builder, validation_error, error_position) = match self {
            Error::Parse {
                expected,
                span,
                reason,
            } => {
                let description = if expected.is_empty() {
                    "Invalid token.".to_string()
                } else {
                    format!("Expected {expected}.")
                };

                (
                    Report::build(
                        ReportKind::Custom("Parsing Error", Color::Yellow),
                        "input",
                        span.1,
                    )
                    .with_message(description)
                    .with_label(
                        Label::new(("input", span.0..span.1))
                            .with_message(reason)
                            .with_color(Color::Red),
                    ),
                    None,
                    span.into(),
                )
            }
            Error::Lex {
                expected,
                span,
                reason,
            } => {
                let description = if expected.is_empty() {
                    "Invalid character.".to_string()
                } else {
                    format!("Expected {expected}.")
                };

                (
                    Report::build(
                        ReportKind::Custom("Lexing Error", Color::Yellow),
                        "input",
                        span.1,
                    )
                    .with_message(description)
                    .with_label(
                        Label::new(("input", span.0..span.1))
                            .with_message(reason)
                            .with_color(Color::Red),
                    ),
                    None,
                    span.into(),
                )
            }
            Error::Runtime { error, position } => (
                Report::build(
                    ReportKind::Custom("Runtime Error", Color::Red),
                    "input",
                    position.1,
                )
                .with_message("An error occured that forced the program to exit.")
                .with_note(
                    "There may be unexpected side-effects because the program could not finish.",
                )
                .with_help(
                    "This is the interpreter's fault. Please submit a bug with this error message.",
                ),
                if let RuntimeError::ValidationFailure(validation_error) = error {
                    Some(validation_error)
                } else {
                    None
                },
                position,
            ),
            Error::Validation { error, position } => (
                Report::build(
                    ReportKind::Custom("Validation Error", Color::Magenta),
                    "input",
                    position.1,
                )
                .with_message("The syntax is valid but this code is not sound.")
                .with_note("This error was detected by the interpreter before running the code."),
                Some(error),
                position,
            ),
        };

        let type_color = Color::Green;
        let identifier_color = Color::Blue;

        if let Some(validation_error) = validation_error {
            match validation_error {
                ValidationError::ExpectedBoolean { actual, position } => {
                    builder.add_label(Label::new(("input", position.0..position.1)).with_message(
                        format!(
                            "Expected {} but got {}.",
                            "boolean".fg(type_color),
                            actual.fg(type_color)
                        ),
                    ));
                }
                ValidationError::ExpectedIntegerOrFloat(position) => {
                    builder.add_label(Label::new(("input", position.0..position.1)).with_message(
                        format!(
                            "Expected {} or {}.",
                            "integer".fg(type_color),
                            "float".fg(type_color)
                        ),
                    ));
                }
                ValidationError::RwLockPoison(_) => todo!(),
                ValidationError::TypeCheck {
                    conflict,
                    actual_position,
                    expected_position: expected_postion,
                } => {
                    let TypeConflict { actual, expected } = conflict;

                    builder = builder.with_message("A type conflict was found.");

                    builder.add_labels([
                        Label::new(("input", expected_postion.0..expected_postion.1)).with_message(
                            format!("Type {} established here.", expected.fg(type_color)),
                        ),
                        Label::new(("input", actual_position.0..actual_position.1))
                            .with_message(format!("Got type {} here.", actual.fg(type_color))),
                    ]);
                }
                ValidationError::VariableNotFound(identifier) => builder.add_label(
                    Label::new(("input", error_position.0..error_position.1)).with_message(
                        format!(
                            "Variable {} does not exist in this context.",
                            identifier.fg(identifier_color)
                        ),
                    ),
                ),
                ValidationError::CannotIndex { r#type, position } => builder.add_label(
                    Label::new(("input", position.0..position.1))
                        .with_message(format!("Cannot index into a {}.", r#type.fg(type_color))),
                ),
                ValidationError::CannotIndexWith {
                    collection_type,
                    collection_position,
                    index_type,
                    index_position,
                } => {
                    builder = builder.with_message(format!(
                        "Cannot index into {} with {}.",
                        collection_type.clone().fg(type_color),
                        index_type.clone().fg(type_color)
                    ));

                    builder.add_labels([
                        Label::new(("input", collection_position.0..collection_position.1))
                            .with_message(format!(
                                "This has type {}.",
                                collection_type.fg(type_color),
                            )),
                        Label::new(("input", index_position.0..index_position.1))
                            .with_message(format!("This has type {}.", index_type.fg(type_color),)),
                    ])
                }
                ValidationError::InterpreterExpectedReturn(_) => todo!(),
                ValidationError::ExpectedFunction { .. } => todo!(),
                ValidationError::ExpectedValue(_) => todo!(),
                ValidationError::PropertyNotFound { .. } => todo!(),
                ValidationError::WrongArguments { .. } => todo!(),
            }
        }

        let mut output = Vec::new();

        builder
            .finish()
            .write_for_stdout(sources([("input", source)]), &mut output)?;

        Ok(output)
    }
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
            reason: error.reason().to_string(),
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
    VariableNotFound(Identifier),
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
