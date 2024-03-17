use std::sync::PoisonError;

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::{prelude::Rich, span::Span};

use crate::{
    abstract_tree::{Identifier, Type},
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
    Runtime(RuntimeError),
    Validation(ValidationError),
}

impl Error {
    pub fn report(&self) -> Report {
        match self {
            Error::Parse { expected, span } => {
                let message = match expected.as_str() {
                    "" => "Invalid character.".to_string(),
                    expected => format!("Expected {expected}."),
                };

                Report::build(ReportKind::Custom("Lexing Error", Color::White), (), span.0)
                    .with_label(Label::new(span.0..span.1).with_message(message))
                    .finish()
            }
            Error::Lex { expected, span } => {
                let message = match expected.as_str() {
                    "" => "Invalid character.".to_string(),
                    expected => format!("Expected {expected}."),
                };

                Report::build(ReportKind::Custom("Lexing Error", Color::White), (), span.0)
                    .with_label(Label::new(span.0..span.1).with_message(message))
                    .finish()
            }
            Error::Runtime(_) => todo!(),
            Error::Validation(validation_error) => {
                let mut report =
                    Report::build(ReportKind::Custom("Validation Error", Color::White), (), 0);

                match validation_error {
                    ValidationError::ExpectedBoolean => {
                        report =
                            report.with_label(Label::new(0..0).with_message("Expected boolean."));
                    }
                    ValidationError::ExpectedIntegerOrFloat => {
                        report = report.with_label(
                            Label::new(0..0).with_message("Expected integer or float."),
                        );
                    }
                    ValidationError::RwLockPoison(_) => todo!(),
                    ValidationError::TypeCheck {
                        conflict,
                        actual_position,
                        expected_position: expected_postion,
                    } => {
                        let TypeConflict { actual, expected } = conflict;

                        report = report.with_labels([
                            Label::new(expected_postion.0..expected_postion.1)
                                .with_message(format!("Type {expected} established here.")),
                            Label::new(actual_position.0..actual_position.1)
                                .with_message(format!("Got type {actual} here.")),
                        ]);
                    }
                    ValidationError::VariableNotFound(identifier) => {
                        report = report
                            .with_label(Label::new(0..0).with_message(format!(
                                "The variable {identifier} does not exist."
                            )));
                    }
                    ValidationError::CannotIndex(_) => todo!(),
                    ValidationError::CannotIndexWith(_, _) => todo!(),
                    ValidationError::InterpreterExpectedReturn => todo!(),
                    ValidationError::ExpectedFunction => todo!(),
                    ValidationError::ExpectedValue => todo!(),
                }

                report.finish()
            }
        }
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

impl From<RuntimeError> for Error {
    fn from(error: RuntimeError) -> Self {
        Error::Runtime(error)
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
    ExpectedBoolean,
    ExpectedFunction,
    ExpectedIntegerOrFloat,
    ExpectedValue,
    InterpreterExpectedReturn,
    RwLockPoison(RwLockPoisonError),
    TypeCheck {
        /// The mismatch that caused the error.
        conflict: TypeConflict,

        /// The position of the item that gave the "actual" type.
        actual_position: (usize, usize),

        /// The position of the item that gave the "expected" type.
        expected_position: (usize, usize),
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
