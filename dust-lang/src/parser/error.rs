use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    Span, Token, Type,
    dust_error::AnnotatedError,
    syntax_tree::{SyntaxId, SyntaxKind},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    // Syntax Errors
    ExpectedToken {
        actual: Token,
        expected: Token,
        position: Span,
    },
    ExpectedMultipleTokens {
        actual: Token,
        expected: &'static [Token],
        position: Span,
    },
    UnexpectedToken {
        actual: Token,
        position: Span,
    },

    // Semantic Errors
    ExpectedItem {
        actual: SyntaxKind,
        position: Span,
    },
    ExpectedStatement {
        actual: SyntaxKind,
        position: Span,
    },
    ExpectedExpression {
        actual: SyntaxKind,
        position: Span,
    },

    // Type Errors
    AdditionTypeMismatch {
        left_type: Type,
        left_position: Span,
        right_type: Type,
        right_position: Span,
        position: Span,
    },
    OperandTypeMismatch {
        operator: Token,
        left_type: Type,
        left_position: Span,
        right_type: Type,
        right_position: Span,
        position: Span,
    },

    // Variable Errors
    UndeclaredVariable {
        identifier: String,
        position: Span,
    },
    DeclarationConflict {
        identifier: String,
        first_declaration: Span,
        second_declaration: Span,
    },

    // Internal Errors
    MissingNode {
        id: SyntaxId,
    },
}

impl AnnotatedError for ParseError {
    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a> {
        match self {
            ParseError::ExpectedToken {
                actual,
                expected,
                position,
            } => {
                let title = "Expected a different token".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.as_usize_range())
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
                            .span(position.as_usize_range())
                            .label(format!(
                                "Found {actual} but expected one of: {expected_list} here"
                            )),
                    ),
                )
            }
            ParseError::UnexpectedToken { position, .. } => {
                let title = "Unexpected token".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.as_usize_range())
                            .label("This token was not expected here"),
                    ),
                )
            }
            ParseError::ExpectedItem { position, .. } => {
                let title = "Expected an item".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.as_usize_range())),
                )
            }
            ParseError::ExpectedStatement { position, .. } => {
                let title = "Expected a statement".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.as_usize_range())),
                )
            }
            ParseError::ExpectedExpression { position, .. } => {
                let title = "Expected an expression".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.as_usize_range())),
                )
            }
            ParseError::AdditionTypeMismatch {
                left_type,
                left_position,
                right_type,
                right_position,
                position,
            } => {
                let title = format!("Cannot add type {left_type} to type {right_type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.as_usize_range()))
                        .annotation(
                            AnnotationKind::Context
                                .span(left_position.as_usize_range())
                                .label(format!("Left operand is of type {left_type}")),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(right_position.as_usize_range())
                                .label(format!("Right operand is of type {right_type}")),
                        ),
                )
            }
            ParseError::OperandTypeMismatch { .. } => todo!(),
            ParseError::UndeclaredVariable { .. } => todo!(),
            ParseError::DeclarationConflict { .. } => todo!(),
            ParseError::MissingNode { .. } => todo!(),
        }
    }
}
