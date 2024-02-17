//! Error and Result types.
//!
//! To deal with errors from dependencies, either create a new error variant
//! or use the ToolFailure variant if the error can only occur inside a tool.
mod runtime_error;
pub(crate) mod rw_lock_error;
mod syntax_error;
mod validation_error;

use colored::Colorize;
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

impl Error {
    /// Create a pretty error report with `lyneate`.
    ///
    /// The `source` argument should be the full source code document that was
    /// used to create this error.
    pub fn create_report(&self, source: &str) -> String {
        match self {
            Error::Syntax(syntax_error) => {
                let report = syntax_error.create_report(source);

                format!(
                    "{}\n{}\n{report}",
                    "Syntax Error".bold().yellow().underline(),
                    "Dust does not recognize this syntax.".dimmed()
                )
            }
            Error::Validation(_) => todo!(),
            Error::Runtime(runtime_error) => {
                let report = runtime_error.create_report(source);

                format!(
                    "{}\n{}\n{report}",
                    "Runtime Error".bold().red().underline(),
                    "This error occured while the program was running.".dimmed()
                )
            }
            Error::ParserCancelled => todo!(),
            Error::Language(_) => todo!(),
        }
    }
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
