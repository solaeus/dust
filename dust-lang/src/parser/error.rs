use annotate_snippets::{AnnotationKind, Group, Level, Snippet};
use smallvec::SmallVec;

use crate::{
    dust_error::AnnotatedError,
    source::{Position, SourceFileId},
    syntax::{SyntaxId, SyntaxKind},
    token::TokenKind,
    r#type::Type,
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
    ExpectedFunction {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedModule {
        identifier: String,
        position: Position,
    },
    PrivateImport {
        identifier: String,
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
    BinaryOperandTypeMismatch {
        operator: TokenKind,
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
        declaration_positions: SmallVec<[Position; 4]>,
    },
    UndeclaredVariable {
        identifier: String,
        position: Position,
    },
    UndeclaredModule {
        identifier: String,
        position: Position,
    },

    // Internal Errors
    MissingNode {
        id: SyntaxId,
    },
    MissingChildren {
        parent_node: SyntaxId,
        children_start: u32,
        children_end: u32,
    },
    MissingPosition {
        identifier: String,
        position: Position,
    },
    MissingSourceFile {
        file_id: SourceFileId,
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
            ParseError::ExpectedFunction { position, .. } => position.file_id,
            ParseError::ExpectedModule { position, .. } => position.file_id,
            ParseError::PrivateImport { position, .. } => position.file_id,
            ParseError::AdditionTypeMismatch { position, .. } => position.file_id,
            ParseError::BinaryOperandTypeMismatch { position, .. } => position.file_id,
            ParseError::ExpectedBooleanCondition {
                condition_position, ..
            } => condition_position.file_id,
            ParseError::InvalidAssignmentTarget { position, .. } => position.file_id,
            ParseError::NegationTypeMismatch { position, .. } => position.file_id,
            ParseError::NotTypeMismatch { position, .. } => position.file_id,
            ParseError::UndeclaredType { position, .. } => position.file_id,
            ParseError::DeclarationConflict {
                second_declaration, ..
            } => second_declaration.file_id,
            ParseError::OutOfScopeVariable { position, .. } => position.file_id,
            ParseError::UndeclaredVariable { position, .. } => position.file_id,
            ParseError::UndeclaredModule { position, .. } => position.file_id,
            ParseError::MissingNode { .. } => SourceFileId::default(),
            ParseError::MissingChildren { .. } => SourceFileId::default(),
            ParseError::MissingPosition { position, .. } => position.file_id,
            ParseError::MissingSourceFile { .. } => SourceFileId::default(),
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
            ParseError::ExpectedModule {
                identifier,
                position,
            } => {
                let title = format!("Expected a module named `{identifier}`");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("No module named `{identifier}` found here")),
                    ),
                )
            }
            ParseError::PrivateImport {
                identifier,
                position,
            } => {
                let title = format!("Cannot import private item `{identifier}`");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("The item `{identifier}` is private")),
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
                declaration_positions,
            } => {
                let title = "Use of out-of-scope variable".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label("This variable is not in scope here"),
                        )
                        .annotations(declaration_positions.iter().map(|decl_pos| {
                            AnnotationKind::Context
                                .span(decl_pos.span.as_usize_range())
                                .label("Variable declared here")
                        })),
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
            ParseError::UndeclaredModule {
                identifier,
                position,
            } => {
                let title = format!("Use of undeclared module `{identifier}`");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("The module `{identifier}` is not declared")),
                    ),
                )
            }
            ParseError::MissingNode { id } => {
                let title = format!("Internal error: Missing syntax node with ID {}", id.0);

                Group::with_title(Level::ERROR.primary_title(title))
            }
            ParseError::MissingChildren {
                parent_node,
                children_start,
                children_end,
            } => {
                let title = format!(
                    "Internal error: Missing children nodes {} to {} for parent node with ID {}",
                    children_start, children_end, parent_node.0
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            ParseError::MissingPosition {
                identifier,
                position,
            } => {
                let title =
                    format!("Internal error: Missing position for identifier `{identifier}`");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("The identifier `{identifier}` is referenced here")),
                    ),
                )
            }
            ParseError::MissingSourceFile { file_id } => {
                let title = format!("Internal error: Missing source file with ID {}", file_id.0);

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
