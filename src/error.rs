use std::sync::PoisonError;

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::{prelude::Rich, span::SimpleSpan};

use crate::{abstract_tree::Type, lexer::Token};

#[derive(Debug, PartialEq)]
pub enum Error {
    Parse {
        expected: String,
        found: Option<String>,
        span: SimpleSpan,
    },
    Lex {
        expected: String,
        found: Option<char>,
        span: SimpleSpan,
    },
    Runtime(RuntimeError),
}

impl Error {
    pub fn report(&self, source: &str) -> Report {
        match self {
            Error::Parse {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Custom("Parsing Error", Color::Red), (), 0).finish(),
            Error::Lex {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Custom("Lexing Error", Color::Red), (), 0)
                .with_label(Label::new(span.start..span.end).with_message(format!(
                    "Exptected {expected} but found {}.",
                    found.unwrap_or(' '),
                )))
                .finish(),
            Error::Runtime(_) => todo!(),
        }
    }
}

impl From<Rich<'_, char>> for Error {
    fn from(error: Rich<'_, char>) -> Self {
        Error::Lex {
            expected: error.expected().map(|error| error.to_string()).collect(),
            found: error.reason().found().map(|c| c.clone()),
            span: error.span().clone(),
        }
    }
}

impl<'src> From<Rich<'_, Token<'src>>> for Error {
    fn from(error: Rich<'_, Token<'src>>) -> Self {
        Error::Parse {
            expected: error.expected().map(|error| error.to_string()).collect(),
            found: error.reason().found().map(|c| c.to_string()),
            span: error.span().clone(),
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
    ExpectedBoolean,
    RwLockPoison(RwLockPoisonError),
    TypeCheck(TypeCheckError),
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
