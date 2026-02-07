use std::borrow::Cow;

use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::{
        Resolver, TypeId,
        resolver::{DeclarationId, DeclarationName, ScopeId},
    },
    dust_error::AnnotatedError,
    source::{Position, Source, SourceFileId},
    syntax::{SyntaxId, SyntaxKind},
    r#type::Type,
};

#[derive(Debug, Clone)]
pub enum CompileError {
    // User Errors
    AmbiguousType {
        name: String,
        position: Position,
    },
    CannotApplyOperator {
        operator: SyntaxKind,
        r#type: Type,
        position: Position,
    },
    CannotInferType {
        position: Position,
    },
    CannotIndex {
        r#type: Type,
        position: Position,
    },
    CannotMutate {
        position: Position,
    },
    ConstantTypeConflict {
        expected: SyntaxKind,
        found: SyntaxKind,
        position: Position,
    },
    DivisionByZero {
        position: Position,
    },
    ExpectedIntegerIndex {
        found: TypeId,
        position: Position,
    },
    ExpectedItem {
        node_kind: SyntaxKind,
        position: Position,
    },
    ExpectedStatement {
        node_kind: SyntaxKind,
        position: Position,
    },
    ExpectedExpression {
        node_kind: SyntaxKind,
        position: Position,
    },
    ExpectedFunction {
        node_kind: SyntaxKind,
        position: Position,
    },
    ExpectedBooleanExpression {
        node_kind: SyntaxKind,
        position: Position,
    },
    TypeConflict {
        expected: Type,
        expected_name: DeclarationName,
        found: Type,
        found_name: DeclarationName,
    },
    UndeclaredVariable {
        name: String,
        position: Position,
    },
    UndeclaredType {
        name: String,
        position: Position,
    },
    Internal(InternalError),
}

impl CompileError {
    fn primary_file_id(&self) -> SourceFileId {
        match self {
            CompileError::AmbiguousType { position, .. }
            | CompileError::CannotApplyOperator { position, .. }
            | CompileError::CannotInferType { position, .. }
            | CompileError::CannotIndex { position, .. }
            | CompileError::CannotMutate { position, .. }
            | CompileError::ConstantTypeConflict { position, .. }
            | CompileError::DivisionByZero { position, .. }
            | CompileError::ExpectedIntegerIndex { position, .. }
            | CompileError::ExpectedItem { position, .. }
            | CompileError::ExpectedStatement { position, .. }
            | CompileError::ExpectedExpression { position, .. }
            | CompileError::ExpectedFunction { position, .. }
            | CompileError::ExpectedBooleanExpression { position, .. }
            | CompileError::UndeclaredVariable { position, .. }
            | CompileError::UndeclaredType { position, .. } => position.file_id,
            CompileError::TypeConflict {
                expected_name,
                found_name,
                ..
            } => match (expected_name, found_name) {
                (DeclarationName::Source(position), _) => position.file_id,
                (_, DeclarationName::Source(position)) => position.file_id,
                _ => SourceFileId::MAIN,
            },
            CompileError::Internal(_) => SourceFileId::MAIN,
        }
    }
}

impl<'a> AnnotatedError<'a> for CompileError {
    type Input = (&'a Source, &'a Resolver);

    fn annotated_error(&'a self, (source, resolver): Self::Input) -> Group<'a> {
        let primary_source_file_str = source.get_file_as_str(self.primary_file_id());

        match self {
            CompileError::DivisionByZero { position } => {
                let title = "Division by zero".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedIntegerIndex { found, position } => {
                let found_type = resolver
                    .get_full_type(*found, source)
                    .map(|r#type| r#type.to_string())
                    .unwrap_or("<invalid type>".to_string());
                let title = format!("Expected an integer index, found {found_type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedItem {
                node_kind,
                position,
            } => {
                let title = format!("Expected an item, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedStatement {
                node_kind,
                position,
            } => {
                let title = format!("Expected a statement, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedBooleanExpression {
                node_kind,
                position,
            } => {
                let title = format!("Expected a boolean expression, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedExpression {
                node_kind,
                position,
            } => {
                let title = format!("Expected an expression, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedFunction {
                node_kind,
                position,
            } => {
                let title = format!("Expected a function, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::UndeclaredVariable { name, position } => {
                let title = format!("Undeclared variable: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of undeclared variable {name} here")),
                    ),
                )
            }
            CompileError::CannotInferType { position } => {
                let title = "Cannot infer type, please provide an explicit type".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::TypeConflict {
                expected,
                expected_name,
                found,
                found_name,
            } => {
                let title = format!("Type mismatch: expected {expected}, found {found}");

                let found_snippet = match found_name {
                    DeclarationName::BuiltIn(name) => Snippet::source(Cow::Borrowed(*name)),
                    DeclarationName::Source(position) => Snippet::source(primary_source_file_str)
                        .annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(format!("Found {found} type here")),
                        ),
                    DeclarationName::External(constant_index) => {
                        let name = resolver
                            .constants
                            .get_string(*constant_index)
                            .unwrap_or("<invalid constant index>");

                        Snippet::source(name)
                    }
                };
                let expected_snippet = match expected_name {
                    DeclarationName::BuiltIn(name) => Snippet::source(Cow::Borrowed(*name)),
                    DeclarationName::Source(position) => Snippet::source(primary_source_file_str)
                        .annotation(
                            AnnotationKind::Context
                                .span(position.span.as_usize_range())
                                .label(format!("The {expected} type was established here.")),
                        ),
                    DeclarationName::External(constant_index) => {
                        let name = resolver
                            .constants
                            .get_string(*constant_index)
                            .unwrap_or("<invalid constant index>");

                        Snippet::source(name)
                    }
                };

                Group::with_title(Level::ERROR.primary_title(title))
                    .elements(vec![found_snippet, expected_snippet])
            }
            CompileError::CannotApplyOperator {
                operator,
                r#type,
                position,
            } => {
                let title = format!("Cannot apply operator {operator} to type {type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Attempted to apply operator {operator} to type {type} here"
                            )),
                    ),
                )
            }
            CompileError::CannotIndex { r#type, position } => {
                let title = format!("Cannot index type {type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Attempted to index type {type} here")),
                    ),
                )
            }
            CompileError::UndeclaredType { name, position } => {
                let title = format!("Undeclared type: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of undeclared type {name} here")),
                    ),
                )
            }
            CompileError::AmbiguousType { name, position } => {
                let title = format!("Ambiguous type: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of ambiguous type {name} here")),
                    ),
                )
            }
            CompileError::ConstantTypeConflict {
                expected,
                found,
                position,
            } => {
                let title = format!("Constant type conflict: expected {expected}, found {found}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found constant of type {found} here, but expected {expected}"
                            )),
                    ),
                )
            }
            CompileError::CannotMutate { position } => {
                let title = "Cannot mutate immutable value".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(primary_source_file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::Internal(internal_error) => {
                let title = format!("Internal compiler error: {internal_error:?}");

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum InternalError {
    MissingSyntaxNode(SyntaxId),
    MissingType(TypeId),
    MissingScope(ScopeId),
    MissingSourceFile(SourceFileId),
    MissingSyntaxTree(SourceFileId),
    MissingScopeBinding(SyntaxId),
    MissingTypeMembers { start_index: u32, count: u32 },
    MissingDeclarationBinding(SyntaxId),
    MissingTypeBinding(SyntaxId),
    MissingDeclarationType(DeclarationId),
    InvalidSyntaxNode(SyntaxKind),
    InvalidTypeNode(TypeId),
    InvalidDeclarationKind(DeclarationId),
    MissingDeclarationMembers(DeclarationId),
    ExpectedFunctionType(TypeId),
}
