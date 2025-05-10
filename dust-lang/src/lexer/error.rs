use crate::{AnnotatedError, Span};

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    ExpectedAsciiHexDigit {
        actual: Option<char>,
        position: usize,
    },
    ExpectedCharacter {
        expected: char,
        actual: char,
        position: usize,
    },
    ExpectedCharacterMultiple {
        expected: &'static [char],
        actual: char,
        position: usize,
    },
    UnexpectedCharacter {
        actual: char,
        position: usize,
    },
    UnexpectedEndOfFile {
        position: usize,
    },
}

impl AnnotatedError for LexError {
    fn title() -> &'static str {
        "Lex Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::ExpectedAsciiHexDigit { .. } => "Expected ASCII hex digit",
            Self::ExpectedCharacter { .. } => "Expected character",
            Self::ExpectedCharacterMultiple { .. } => "Expected one of multiple characters",
            Self::UnexpectedCharacter { .. } => "Unexpected character",
            Self::UnexpectedEndOfFile { .. } => "Unexpected end of file",
        }
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        match self {
            Self::ExpectedAsciiHexDigit { actual, position } => {
                vec![(
                    format!(
                        "Expected an ASCII hex digit (0-9, A-F, a-f), but found `{}`",
                        actual.map_or("end of input".to_string(), |c| c.to_string())
                    ),
                    Span(*position, *position + 1),
                )]
            }
            Self::ExpectedCharacter {
                expected,
                actual,
                position,
            } => {
                vec![(
                    format!("Expected character `{expected}`, but found `{actual}`"),
                    Span(*position, *position + 1),
                )]
            }
            Self::ExpectedCharacterMultiple {
                expected,
                actual,
                position,
            } => {
                vec![(
                    format!("Expected one of the characters `{expected:?}`, but found `{actual}`"),
                    Span(*position, *position + 1),
                )]
            }
            Self::UnexpectedCharacter { actual, position } => {
                vec![(
                    format!("Unexpected character `{actual}`"),
                    Span(*position, *position + 1),
                )]
            }
            Self::UnexpectedEndOfFile { position } => {
                vec![(
                    "Unexpected end of file while lexing".to_string(),
                    Span(*position, *position),
                )]
            }
        }
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        match self {
            Self::ExpectedAsciiHexDigit { position, .. } => {
                vec![(
                    "Ensure the input contains valid hexadecimal digits (0-9, A-F, a-f)"
                        .to_string(),
                    Span(*position, *position + 1),
                )]
            }
            Self::ExpectedCharacter {
                expected, position, ..
            } => {
                vec![(
                    format!("Insert the expected character `{expected}` here"),
                    Span(*position, *position + 1),
                )]
            }
            Self::ExpectedCharacterMultiple {
                expected, position, ..
            } => {
                vec![(
                    format!("Insert one of the expected characters `{expected:?}` here"),
                    Span(*position, *position + 1),
                )]
            }
            Self::UnexpectedCharacter { position, .. } => {
                vec![(
                    "Remove or replace the unexpected character".to_string(),
                    Span(*position, *position + 1),
                )]
            }
            Self::UnexpectedEndOfFile { position } => {
                vec![(
                    "Ensure the input is complete and properly terminated".to_string(),
                    Span(*position, *position),
                )]
            }
        }
    }
}
