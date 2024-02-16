//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.
mod runtime_error;
pub(crate) mod rw_lock_error;
mod syntax_error;
mod validation_error;

pub use runtime_error::RuntimeError;
pub use syntax_error::SyntaxError;
pub use validation_error::ValidationError;

use tree_sitter::LanguageError;

use std::fmt::{self, Formatter};

#[derive(PartialEq)]
pub enum Error {
    Syntax(SyntaxError),

    Validation(ValidationError),

    Runtime(RuntimeError),

    ParserCancelled,

    Language(LanguageError),
}

impl From<SyntaxError> for Error {
    fn from(error: SyntaxError) -> Self {
        Error::Syntax(error)
    }
}

impl From<ValidationError> for Error {
    fn from(error: ValidationError) -> Self {
        Error::Validation(error)
    }
}

impl From<RuntimeError> for Error {
    fn from(error: RuntimeError) -> Self {
        Error::Runtime(error)
    }
}

impl From<LanguageError> for Error {
    fn from(error: LanguageError) -> Self {
        Error::Language(error)
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Error::*;

        match self {
            Syntax(error) => write!(f, "{error}"),
            Validation(error) => write!(f, "{error}"),
            Runtime(error) => write!(f, "{error}"),
            ParserCancelled => write!(f, "Parsing was cancelled because the parser took too long."),
            Language(_error) => write!(f, "Parser failed to load language grammar."),
        }
    }
}
