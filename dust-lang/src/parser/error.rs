use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    Span, Token, Type,
    dust_error::AnnotatedError,
    resolver::ScopeId,
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
    BinaryOperandTypeMismatch {
        operator: Token,
        left_type: Type,
        left_position: Span,
        right_type: Type,
        right_position: Span,
        position: Span,
    },
    NegationTypeMismatch {
        operand_type: Type,
        operand_position: Span,
        position: Span,
    },
    NotTypeMismatch {
        operand_type: Type,
        operand_position: Span,
        position: Span,
    },
    UndeclaredType {
        identifier: String,
        position: Span,
    },

    // Variable Errors
    DeclarationConflict {
        identifier: String,
        first_declaration: Span,
        second_declaration: Span,
    },
    UndeclaredVariable {
        identifier: String,
        position: Span,
    },

    // Internal Errors
    MissingNode {
        id: SyntaxId,
    },
    MissingScope {
        id: ScopeId,
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
            ParseError::BinaryOperandTypeMismatch {
                operator,
                left_type,
                left_position,
                right_type,
                right_position,
                position,
            } => {
                let title = format!(
                    "Cannot apply operator {operator} to types {left_type} and {right_type}"
                );

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
            ParseError::NegationTypeMismatch {
                operand_type,
                operand_position,
                position,
            } => {
                let title = format!("Cannot negate type {operand_type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.as_usize_range()))
                        .annotation(
                            AnnotationKind::Context
                                .span(operand_position.as_usize_range())
                                .label(format!("Operand is of type {operand_type}")),
                        ),
                )
            }
            ParseError::NotTypeMismatch {
                operand_type,
                operand_position,
                position,
            } => {
                let title = format!("Cannot apply 'not' to type {operand_type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.as_usize_range()))
                        .annotation(
                            AnnotationKind::Context
                                .span(operand_position.as_usize_range())
                                .label(format!("Operand is of type {operand_type}")),
                        ),
                )
            }
            ParseError::UndeclaredType {
                identifier,
                position,
            } => {
                let title = format!("Use of undeclared type `{identifier}`");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.as_usize_range())
                            .label(format!("The type `{identifier}` is not declared")),
                    ),
                )
            }
            ParseError::DeclarationConflict {
                identifier,
                first_declaration,
                second_declaration,
            } => {
                let title = format!("Declaration conflict for variable `{identifier}`");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(
                            AnnotationKind::Primary
                                .span(second_declaration.as_usize_range())
                                .label(format!(
                                    "The variable `{identifier}` is declared here again"
                                )),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(first_declaration.as_usize_range())
                                .label(format!("First declaration of `{identifier}` is here")),
                        ),
                )
            }
            ParseError::UndeclaredVariable {
                identifier,
                position,
            } => {
                let title = format!("Use of undeclared variable `{identifier}`");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.as_usize_range())
                            .label(format!("The variable `{identifier}` is not declared")),
                    ),
                )
            }
            ParseError::MissingNode { id } => {
                let title = format!("Internal error: Missing syntax node with ID {}", id.0);

                Group::with_title(Level::ERROR.primary_title(title))
            }
            ParseError::MissingScope { id } => {
                let title = format!("Internal error: Missing scope with ID {}", id.0);

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
