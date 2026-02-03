use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    dust_error::AnnotatedError,
    source::{Position, SourceFileId},
    syntax::{SyntaxId, SyntaxKind},
    token::TokenKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidUtf8 {
        position: Position,
    },

    // Syntax Errors
    ExpectedToken {
        actual: TokenKind,
        expected: TokenKind,
        position: Position,
    },
    ExpectedMultipleTokens {
        actual: TokenKind,
        expected: &'static [TokenKind],
        position: Position,
    },
    UnexpectedToken {
        actual: TokenKind,
        position: Position,
    },

    // Semantic Errors
    ExpectedItem {
        actual: SyntaxKind,
        position: Position,
    },
    ExpectedStatement {
        actual: SyntaxKind,
        position: Position,
    },
    ExpectedExpression {
        actual: SyntaxKind,
        position: Position,
    },

    // Internal Errors
    MissingNode {
        id: SyntaxId,
    },
}

impl AnnotatedError for ParseError {
    fn file_id(&self) -> SourceFileId {
        match self {
            ParseError::InvalidUtf8 { position } => position.file_id,
            ParseError::ExpectedToken { position, .. } => position.file_id,
            ParseError::ExpectedMultipleTokens { position, .. } => position.file_id,
            ParseError::UnexpectedToken { position, .. } => position.file_id,
            ParseError::ExpectedItem { position, .. } => position.file_id,
            ParseError::ExpectedStatement { position, .. } => position.file_id,
            ParseError::ExpectedExpression { position, .. } => position.file_id,
            ParseError::MissingNode { .. } => SourceFileId::default(),
        }
    }

    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a> {
        match self {
            ParseError::InvalidUtf8 { position } => {
                let title = "Invalid UTF-8 sequence".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label("This is not valid UTF-8"),
                    ),
                )
            }
            ParseError::ExpectedToken {
                actual,
                expected,
                position,
            } => {
                let title = "Expected a different token".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Found {actual} but expected {expected} here")),
                    ),
                )
            }
            ParseError::ExpectedMultipleTokens {
                actual,
                expected,
                position,
            } => {
                let title = "Expected a different token".to_string();
                let expected_list = expected
                    .iter()
                    .map(|token| format!("{token}"))
                    .collect::<Vec<String>>()
                    .join(", ");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found {actual} but expected one of: {expected_list} here"
                            )),
                    ),
                )
            }
            ParseError::UnexpectedToken { position, actual } => {
                let title = "Unexpected token".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("{actual} was not expected here")),
                    ),
                )
            }
            ParseError::ExpectedItem { position, .. } => {
                let title = "Expected an item".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            ParseError::ExpectedStatement { position, .. } => {
                let title = "Expected a statement".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            ParseError::ExpectedExpression { position, .. } => {
                let title = "Expected an expression".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            ParseError::MissingNode { id } => {
                let title = format!("Internal error: Missing syntax node with ID {}", id.0);

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
