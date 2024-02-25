use std::sync::PoisonError;

use chumsky::prelude::Rich;

use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Error<'src> {
    Parse(Vec<Rich<'src, Token<'src>>>),
    Lex(Vec<Rich<'src, char>>),
    Runtime(RuntimeError),
}

impl<'src> From<Vec<Rich<'src, Token<'src>>>> for Error<'src> {
    fn from(errors: Vec<Rich<'src, Token<'src>>>) -> Self {
        Error::Parse(errors)
    }
}

impl<'src> From<Vec<Rich<'src, char>>> for Error<'src> {
    fn from(errors: Vec<Rich<'src, char>>) -> Self {
        Error::Lex(errors)
    }
}

impl<'src> From<RuntimeError> for Error<'src> {
    fn from(error: RuntimeError) -> Self {
        Error::Runtime(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    RwLockPoison(RwLockPoisonError),
    ExpectedBoolean,
}

impl From<RwLockPoisonError> for RuntimeError {
    fn from(error: RwLockPoisonError) -> Self {
        RuntimeError::RwLockPoison(error)
    }
}

#[derive(Debug, PartialEq)]
pub struct RwLockPoisonError;

impl<T> From<PoisonError<T>> for RwLockPoisonError {
    fn from(_: PoisonError<T>) -> Self {
        RwLockPoisonError
    }
}
