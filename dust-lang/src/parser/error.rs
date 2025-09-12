use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    Position, Token, Type,
    dust_error::AnnotatedError,
    resolver::{DeclarationId, DeclarationKind, ScopeId, TypeId},
    syntax_tree::{SyntaxId, SyntaxKind},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    // Syntax Errors
    ExpectedToken {
        actual: Token,
        expected: Token,
        position: Position,
    },
    ExpectedMultipleTokens {
        actual: Token,
        expected: &'static [Token],
        position: Position,
    },
    UnexpectedToken {
        actual: Token,
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
    ExpectedFunction {
        found: SyntaxKind,
        position: Position,
    },

    // Type Errors
    AdditionTypeMismatch {
        left_type: Type,
        left_position: Position,
        right_type: Type,
        right_position: Position,
        position: Position,
    },
    AssignmentToImmutable {
        found: DeclarationKind,
        position: Position,
    },
    BinaryOperandTypeMismatch {
        operator: Token,
        left_type: Type,
        left_position: Position,
        right_type: Type,
        right_position: Position,
        position: Position,
    },
    ExpectedBooleanCondition {
        condition_type: Type,
        condition_position: Position,
    },
    InvalidAssignmentTarget {
        found: SyntaxKind,
        position: Position,
    },
    NegationTypeMismatch {
        operand_type: Type,
        operand_position: Position,
        position: Position,
    },
    NotTypeMismatch {
        operand_type: Type,
        operand_position: Position,
        position: Position,
    },
    UndeclaredType {
        identifier: String,
        position: Position,
    },

    // Variable Errors
    DeclarationConflict {
        identifier: String,
        first_declaration: Position,
        second_declaration: Position,
    },
    OutOfScopeVariable {
        position: Position,
        declaration_position: Position,
    },
    UndeclaredVariable {
        identifier: String,
        position: Position,
    },

    // Internal Errors
    MissingNode {
        id: SyntaxId,
    },
    MissingScope {
        id: ScopeId,
    },
    MissingDeclaration {
        id: DeclarationId,
    },
    MissingType {
        id: TypeId,
    },
}

impl AnnotatedError for ParseError {
    fn file_index(&self) -> usize {
        (match self {
            ParseError::ExpectedToken { position, .. } => position.file_index,
            ParseError::ExpectedMultipleTokens { position, .. } => position.file_index,
            ParseError::UnexpectedToken { position, .. } => position.file_index,
            ParseError::ExpectedItem { position, .. } => position.file_index,
            ParseError::ExpectedStatement { position, .. } => position.file_index,
            ParseError::ExpectedExpression { position, .. } => position.file_index,
            ParseError::ExpectedFunction { position, .. } => position.file_index,
            ParseError::AdditionTypeMismatch { position, .. } => position.file_index,
            ParseError::AssignmentToImmutable { position, .. } => position.file_index,
            ParseError::BinaryOperandTypeMismatch { position, .. } => position.file_index,
            ParseError::ExpectedBooleanCondition {
                condition_position, ..
            } => condition_position.file_index,
            ParseError::InvalidAssignmentTarget { position, .. } => position.file_index,
            ParseError::NegationTypeMismatch { position, .. } => position.file_index,
            ParseError::NotTypeMismatch { position, .. } => position.file_index,
            ParseError::UndeclaredType { position, .. } => position.file_index,
            ParseError::DeclarationConflict {
                second_declaration, ..
            } => second_declaration.file_index,
            ParseError::OutOfScopeVariable { position, .. } => position.file_index,
            ParseError::UndeclaredVariable { position, .. } => position.file_index,
            ParseError::MissingNode { .. } => 0,
            ParseError::MissingScope { .. } => 0,
            ParseError::MissingDeclaration { .. } => 0,
            ParseError::MissingType { .. } => 0,
        }) as usize
    }

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
            ParseError::UnexpectedToken { position, .. } => {
                let title = "Unexpected token".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label("This token was not expected here"),
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
            ParseError::ExpectedFunction { found, position } => {
                let title = format!("Expected a function, found {found}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("This {found} is not a function")),
                    ),
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
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range()))
                        .annotation(
                            AnnotationKind::Context
                                .span(left_position.span.as_usize_range())
                                .label(format!("Left operand is of type {left_type}")),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(right_position.span.as_usize_range())
                                .label(format!("Right operand is of type {right_type}")),
                        ),
                )
            }
            ParseError::AssignmentToImmutable { found, position } => {
                let title = format!("Cannot assign to immutable {found}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("This {found} is not mutable")),
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
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range()))
                        .annotation(
                            AnnotationKind::Context
                                .span(left_position.span.as_usize_range())
                                .label(format!("Left operand is of type {left_type}")),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(right_position.span.as_usize_range())
                                .label(format!("Right operand is of type {right_type}")),
                        ),
                )
            }
            ParseError::ExpectedBooleanCondition {
                condition_type,
                condition_position,
            } => {
                let title = format!("Expected a boolean condition, found type {condition_type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(condition_position.span.as_usize_range())
                            .label(format!("Condition is of type {condition_type}")),
                    ),
                )
            }
            ParseError::InvalidAssignmentTarget { found, position } => {
                let title = format!("Invalid assignment target: found {found}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("{found} cannot be assigned to here")),
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
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range()))
                        .annotation(
                            AnnotationKind::Context
                                .span(operand_position.span.as_usize_range())
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
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range()))
                        .annotation(
                            AnnotationKind::Context
                                .span(operand_position.span.as_usize_range())
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
                            .span(position.span.as_usize_range())
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
                                .span(second_declaration.span.as_usize_range())
                                .label(format!(
                                    "The variable `{identifier}` is declared here again"
                                )),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(first_declaration.span.as_usize_range())
                                .label(format!("First declaration of `{identifier}` is here")),
                        ),
                )
            }
            ParseError::OutOfScopeVariable {
                position,
                declaration_position,
            } => {
                let title = "Use of out-of-scope variable".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label("This variable is used out of its scope"),
                        )
                        .annotation(
                            AnnotationKind::Context
                                .span(declaration_position.span.as_usize_range())
                                .label("The variable is declared here"),
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
                            .span(position.span.as_usize_range())
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
            ParseError::MissingDeclaration { id } => {
                let title = format!(
                    "Internal error: Missing scope for declaration with ID {}",
                    id.0
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            ParseError::MissingType { id } => {
                let title = format!("Internal error: Missing type with ID {}", id.0);

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
