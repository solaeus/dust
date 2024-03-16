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
    Validation {
        error: ValidationError,
        span: (usize, usize),
    },
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
            Error::Validation { error, span } => {
                let mut report = Report::build(
                    ReportKind::Custom("Validation Error", Color::White),
                    (),
                    span.0,
                );

                match error {
                    ValidationError::ExpectedBoolean => {
                        report = report.with_label(
                            Label::new(span.0..span.1).with_message("Expected boolean."),
                        );
                    }
                    ValidationError::ExpectedIntegerOrFloat => {
                        report = report.with_label(
                            Label::new(span.0..span.1).with_message("Expected integer or float."),
                        );
                    }
                    ValidationError::RwLockPoison(_) => todo!(),
                    ValidationError::TypeCheck(TypeCheckError { actual, expected }) => {
                        report = report.with_label(Label::new(span.0..span.1).with_message(
                            format!("Type error. Expected {expected} but got {actual}."),
                        ));
                    }
                    ValidationError::VariableNotFound(identifier) => {
                        report = report
                            .with_label(Label::new(span.0..span.1).with_message(format!(
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
    TypeCheck(TypeCheckError),
    VariableNotFound(Identifier),
}

impl From<RwLockPoisonError> for ValidationError {
    fn from(error: RwLockPoisonError) -> Self {
        ValidationError::RwLockPoison(error)
    }
}

impl From<TypeCheckError> for ValidationError {
    fn from(error: TypeCheckError) -> Self {
        ValidationError::TypeCheck(error)
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
pub struct TypeCheckError {
    pub actual: Type,
    pub expected: Type,
}
