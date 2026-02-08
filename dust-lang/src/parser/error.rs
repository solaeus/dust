use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    dust_error::AnnotatedError,
    source::{Position, Source, SourceFileId},
    syntax::SyntaxKind,
    token::TokenKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    // Lexer Errors
    InvalidUtf8 {
        position: Position,
    },

    // Syntax Errors
    ExpectedToken {
        found: TokenKind,
        expected: TokenKind,
        position: Position,
    },
    ExpectedMultipleTokens {
        found: TokenKind,
        expected: &'static [TokenKind],
        position: Position,
    },
    UnexpectedToken {
        found: TokenKind,
        position: Position,
    },

    // Semantic Errors
    ExpectedItem {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedStatement {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedExpression {
        found: Option<SyntaxKind>,
        position: Position,
    },
}

impl ParseError {
    fn file_id(&self) -> SourceFileId {
        match self {
            ParseError::InvalidUtf8 { position }
            | ParseError::ExpectedToken { position, .. }
            | ParseError::ExpectedMultipleTokens { position, .. }
            | ParseError::UnexpectedToken { position, .. }
            | ParseError::ExpectedItem { position, .. }
            | ParseError::ExpectedStatement { position, .. }
            | ParseError::ExpectedExpression { position, .. } => position.file_id,
        }
    }
}

impl<'a> AnnotatedError<'a> for ParseError {
    type Input = &'a Source;

    fn annotated_error(&self, source: Self::Input) -> Group<'a> {
        match self {
            ParseError::InvalidUtf8 { position } => {
                let title = "Invalid UTF-8 sequence".to_string();
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label("This is not valid UTF-8"),
                    ),
                )
            }
            ParseError::ExpectedToken {
                found: actual,
                expected,
                position,
            } => {
                let title = "Expected a different token".to_string();
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Found {actual} but expected {expected} here")),
                    ),
                )
            }
            ParseError::ExpectedMultipleTokens {
                found: actual,
                expected,
                position,
            } => {
                let title = "Expected a different token".to_string();
                let file_str = source.get_file(position.file_id).full_source_str();
                let expected_list = expected
                    .iter()
                    .enumerate()
                    .map(|(index, token)| {
                        let is_last = index == expected.len() - 1;

                        if is_last && expected.len() > 1 {
                            format!("or {token}")
                        } else {
                            format!("{token}, ")
                        }
                    })
                    .collect::<String>();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found {actual} but expected one of: {expected_list} here"
                            )),
                    ),
                )
            }
            ParseError::UnexpectedToken { position, found } => {
                let title = "Unexpected token".to_string();
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("{found} was not expected here")),
                    ),
                )
            }
            ParseError::ExpectedItem { position, found } => {
                let title = format!("Expected an item, but found {found}");
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            ParseError::ExpectedStatement { position, found } => {
                let title = format!("Expected a statement, but found {found}");
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            ParseError::ExpectedExpression { position, found } => {
                let title = match found {
                    Some(found) => format!("Expected an expression, but found {found}"),
                    None => "Expected an expression".to_string(),
                };
                let file_str = source.get_file(position.file_id).full_source_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
        }
    }
}
