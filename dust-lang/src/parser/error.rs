use std::fmt::{self, Display, Formatter};

use crate::{
    Span, Token, Type,
    dust_error::{AnnotatedError, ErrorMessage},
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

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::ExpectedItem { actual, position } => {
                write!(f, "Expected item, found {actual} at {position}")
            }
            ParseError::ExpectedStatement { actual, position } => {
                write!(f, "Expected statement, found {actual} at {position}")
            }
            ParseError::ExpectedExpression {
                actual: found,
                position,
            } => {
                write!(f, "Expected expression, found {found} at {position}")
            }
            ParseError::ExpectedToken {
                actual,
                expected,
                position,
            } => {
                write!(
                    f,
                    "Found '{expected}' at {position} but expected '{actual}'",
                )
            }
            ParseError::ExpectedMultipleTokens {
                expected,
                actual,
                position,
            } => {
                write!(
                    f,
                    "Found \"{actual}\" at {position} but expected one of the following: ",
                )?;

                for (i, expected_token) in expected.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    } else if i == expected.len() - 1 {
                        write!(f, "or ")?;
                    }

                    write!(f, "\"{expected_token}\"")?;
                }

                write!(f, ".")
            }
            ParseError::UnexpectedToken {
                actual: found,
                position,
            } => {
                write!(f, "Unexpected token {found} at {position}")
            }

            ParseError::AdditionTypeMismatch {
                left_type,
                right_type,
                ..
            } => {
                write!(f, "Cannot add {left_type} and {right_type}")
            }
            ParseError::OperandTypeMismatch {
                operator,
                left_type,
                right_type,
                ..
            } => {
                write!(
                    f,
                    "The '{operator}' requires two values of the same type, found: {left_type} {operator} {right_type}"
                )
            }

            ParseError::UndeclaredVariable { identifier, .. } => {
                write!(f, "Undeclared variable \"{identifier}\"")
            }
            ParseError::DeclarationConflict { identifier, .. } => {
                write!(
                    f,
                    "Variable \"{identifier}\" is already declared in this scope"
                )
            }

            ParseError::MissingNode { id } => {
                write!(f, "Internal Error: Missing node with ID {}", id.0)
            }
        }
    }
}

impl AnnotatedError for ParseError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "Parsing Error";

        match self {
            ParseError::ExpectedToken { position, .. } => ErrorMessage {
                title,
                description: "Expected a specific token",
                detail_snippets: vec![(self.to_string(), *position)],
                help_snippet: None,
            },
            ParseError::ExpectedMultipleTokens { position, .. } => ErrorMessage {
                title: "Expected Multiple Tokens",
                description: "Expected one of several tokens",
                detail_snippets: vec![(self.to_string(), *position)],
                help_snippet: None,
            },
            ParseError::UnexpectedToken { position, .. } => ErrorMessage {
                title: "Unexpected Token",
                description: "Unexpected token",
                detail_snippets: vec![("Found here".to_string(), *position)],
                help_snippet: None,
            },

            ParseError::ExpectedItem { actual, position } => ErrorMessage {
                title,
                description: "Expected an item",
                detail_snippets: vec![(
                    format!("This is a {actual}, which cannot be used here."),
                    *position,
                )],
                help_snippet: None,
            },
            ParseError::ExpectedStatement { actual, position } => ErrorMessage {
                title,
                description: "Expected a statement",
                detail_snippets: vec![(
                    format!("This is a {actual}, which cannot be used here."),
                    *position,
                )],
                help_snippet: None,
            },
            ParseError::ExpectedExpression {
                actual: found,
                position,
            } => ErrorMessage {
                title,
                description: "Expected an expression",
                detail_snippets: vec![(
                    format!("This is a {found}, which cannot be used here."),
                    *position,
                )],
                help_snippet: None,
            },

            ParseError::AdditionTypeMismatch {
                left_type,
                left_position,
                right_type,
                right_position,
                position,
            } => ErrorMessage {
                title,
                description: "Cannot add these two types together",
                detail_snippets: vec![
                    (
                        "The '+' operator requires both sides to be numbers of the same type or strings/characters.".to_string(),
                        *position,
                    ),
                    (
                        format!("The left side has type {left_type}"),
                        *left_position,
                    ),
                    (
                        format!("The right side has type {right_type}"),
                        *right_position,
                    ),
                ],
                help_snippet: None,
            },
            ParseError::OperandTypeMismatch {
                operator,
                left_type,
                left_position,
                right_type,
                right_position,
                position,
            } => ErrorMessage {
                title,
                description: "Type conflict",
                detail_snippets: vec![
                    (
                        format!("The '{operator}' operator requires both sides to be of the same type."),
                        *position,
                    ),
                    (
                        format!("The left side has type {left_type}"),
                        *left_position,
                    ),
                    (
                        format!("The right side has type {right_type}"),
                        *right_position,
                    ),
                ],
                help_snippet: None,
            },

            ParseError::UndeclaredVariable { identifier, position } => ErrorMessage {
                title,
                description: "Use of undeclared variable",
                detail_snippets: vec![(
                    format!("The variable \"{identifier}\" is used here but has not been declared."),
                    *position,
                )],
                help_snippet: None,
            },
            ParseError::DeclarationConflict {
                identifier,
                first_declaration,
                second_declaration,
            } => ErrorMessage {
                title,
                description: "Variable declaration conflict",
                detail_snippets: vec![
                    (
                        format!("\"{identifier}\" is first declared here."),
                        *first_declaration,
                    ),
                    (
                        format!("\"{identifier}\" is redeclared here, in the same scope."),
                        *second_declaration,
                    ),
                ],
                help_snippet: None,
            },

            ParseError::MissingNode { id } => ErrorMessage {
                title,
                description: "A node was expected but is missing from the syntax tree.",
                detail_snippets: vec![(
                    format!("Node with ID {} is missing.", id.0),
                    Span::default(),
                )],
                help_snippet: Some("This is a bug in the parser".to_string()),
            },
        }
    }
}
