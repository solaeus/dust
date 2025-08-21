use std::fmt::{self, Display, Formatter};

use crate::{
    Span,
    dust_error::{AnnotatedError, ErrorMessage},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexError {
    ExpectedCharacter {
        expected: char,
        actual: Option<char>,
        position: usize,
    },
    ExpectedMultipleCharacters {
        expected: &'static [char],
        actual: Option<char>,
        position: usize,
    },
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LexError::ExpectedCharacter {
                expected,
                actual,
                position,
            } => {
                write!(
                    f,
                    "Found '{expected}' at {position} but expected '{}'",
                    actual
                        .map(|char| char.to_string())
                        .unwrap_or_else(|| "EOF".to_string())
                )
            }
            LexError::ExpectedMultipleCharacters {
                expected,
                actual,
                position,
            } => {
                write!(
                    f,
                    "Found \"{}\" at {position} but expected one of the following: ",
                    actual
                        .map(|char| char.to_string())
                        .unwrap_or_else(|| "EOF".to_string())
                )?;

                for (i, expected) in expected.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "\"{expected}\"")?;
                }

                write!(f, ".")
            }
        }
    }
}

impl AnnotatedError for LexError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "Lexing Error";

        match self {
            LexError::ExpectedCharacter { position, .. } => ErrorMessage {
                title,
                description: "Expected a specific character",
                detail_snippets: vec![(self.to_string(), Span::new(*position, *position + 1))],
                help_snippet: None,
            },
            LexError::ExpectedMultipleCharacters { position, .. } => ErrorMessage {
                title,
                description: "Expected one of several characters",
                detail_snippets: vec![(self.to_string(), Span::new(*position, *position + 1))],
                help_snippet: None,
            },
        }
    }
}
