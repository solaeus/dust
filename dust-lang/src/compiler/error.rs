use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::{
        Resolver, TypeId,
        resolver::{DeclarationId, DeclarationMembers, ScopeId, TypeMembers},
    },
    dust_error::AnnotatedError,
    source::{Position, Source, SourceFileId},
    syntax::{SyntaxId, SyntaxKind},
    r#type::Type,
};

const INVALID_TYPE: &str = "<invalid_type>";

#[derive(Debug, Clone)]
pub enum CompileError {
    Internal(InternalError),

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
        type_id: TypeId,
        position: Option<Position>,
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
        found: TypeId,
        node_kind: SyntaxKind,
        position: Position,
    },
    TypeConflict {
        expected_type: TypeId,
        expected_position: Option<Position>,
        found_type: TypeId,
        found_position: Position,
    },
    UndeclaredVariable {
        position: Position,
    },
    UndeclaredType {
        name: String,
        position: Position,
    },
    ExpectedFunctionType {
        found: TypeId,
        position: Position,
    },
    ExpectedArguments {
        function_type: TypeId,
        expected_count: usize,
        found_count: usize,
        found_position: Position,
    },
}

impl<'a> AnnotatedError<'a> for CompileError {
    type Input = (&'a Source, &'a Resolver);

    fn annotated_error(&'a self, (source, resolver): Self::Input) -> Group<'a> {
        match self {
            CompileError::DivisionByZero { position } => {
                let title = "Division by zero".to_string();
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedIntegerIndex { found, position } => {
                let found_type = resolver
                    .get_full_type(*found, source)
                    .map(|r#type| r#type.to_string())
                    .unwrap_or("<invalid type>".to_string());
                let title = format!("Expected an integer index, found {found_type}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedItem {
                node_kind,
                position,
            } => {
                let title = format!("Expected an item, found {node_kind}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedStatement {
                node_kind,
                position,
            } => {
                let title = format!("Expected a statement, found {node_kind}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedBooleanExpression {
                found: found_type_id,
                node_kind,
                position,
            } => {
                let title = format!("Expected a boolean expression");
                let file_str = source.get_file_as_str(position.file_id);
                let found_type = resolver
                    .get_full_type(*found_type_id, source)
                    .map(|r#type| r#type.to_string())
                    .unwrap_or_else(|| {
                        let internal_error = InternalError::MissingType(*found_type_id);

                        format!("{internal_error:#?}>")
                    });

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range()).label(format!(
                            "Expected a boolean expression here, but found this {node_kind} with type {found_type}."
                        ))),
                )
            }
            CompileError::ExpectedExpression {
                node_kind,
                position,
            } => {
                let title = format!("Expected an expression, found {node_kind}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedFunction {
                node_kind,
                position,
            } => {
                let title = format!("Expected a function, found {node_kind}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::UndeclaredVariable { position } => {
                let variable_str = source.get_source_str(position);
                let file_str = source.get_file_as_str(position.file_id);
                let title = format!("Undeclared variable: {variable_str}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "\"{variable_str}\" was used here, but it was not declared in this scope."
                            )),
                    ),
                )
            }
            CompileError::CannotInferType { type_id, position } => {
                let title = "Cannot infer type".to_string();
                let type_string = resolver
                    .get_full_type(*type_id, source)
                    .map_or_else(|| "<invalid_type>".to_string(), |r#type| r#type.to_string());
                let message = format!("Cannot infer type for \"{type_string}\".");

                if let Some(position) = position {
                    let file_str = source.get_file_as_str(position.file_id);

                    Group::with_title(Level::ERROR.primary_title(title)).element(
                        Snippet::source(file_str).annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(message),
                        ),
                    )
                } else {
                    Group::with_title(Level::ERROR.primary_title(title))
                        .element(Level::ERROR.message(message))
                }
            }
            CompileError::TypeConflict {
                expected_type,
                expected_position,
                found_type,
                found_position,
            } => {
                let title = format!("Type conflict");

                let expected_type_string = resolver
                    .get_full_type(*expected_type, source)
                    .map_or_else(|| INVALID_TYPE.to_string(), |r#type| r#type.to_string());
                let found_type_string = resolver
                    .get_full_type(*found_type, source)
                    .map_or_else(|| INVALID_TYPE.to_string(), |r#type| r#type.to_string());

                if let Some(expected_position) = expected_position {
                    let expected_file_str = source.get_file_as_str(expected_position.file_id);
                    let found_file_str = source.get_file_as_str(found_position.file_id);

                    Group::with_title(Level::ERROR.primary_title(title)).elements([
                        Snippet::source(found_file_str).annotation(
                            AnnotationKind::Primary
                                .span(found_position.span.as_usize_range())
                                .label(format!("Found {found_type_string} here.")),
                        ),
                        Snippet::source(expected_file_str).annotation(
                            AnnotationKind::Context
                                .span(expected_position.span.as_usize_range())
                                .label(format!(
                                    "Type {expected_type_string} was established here."
                                )),
                        ),
                    ])
                } else {
                    let file_str = source.get_file_as_str(found_position.file_id);

                    Group::with_title(Level::ERROR.primary_title(title))
                        .element(
                            Snippet::source(file_str).annotation(
                                AnnotationKind::Primary
                                    .span(found_position.span.as_usize_range())
                                    .label(format!("Found {found_type_string} here.")),
                            ),
                        )
                        .element(Level::ERROR.message(format!("Expected {expected_type_string}.")))
                }
            }
            CompileError::CannotApplyOperator {
                operator,
                r#type,
                position,
            } => {
                let title = format!("Cannot apply operator {operator} to type {type}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
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
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Attempted to index type {type} here")),
                    ),
                )
            }
            CompileError::UndeclaredType { name, position } => {
                let title = format!("Undeclared type: {name}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of undeclared type {name} here")),
                    ),
                )
            }
            CompileError::AmbiguousType { name, position } => {
                let title = format!("Ambiguous type: {name}");
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
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
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
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
                let file_str = source.get_file_as_str(position.file_id);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::Internal(internal_error) => {
                let title = format!("Internal compiler error: {internal_error:?}");

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::ExpectedFunctionType { found, position } => {
                let title = "Expected a function type";
                let file_str = source.get_file_as_str(position.file_id);
                let found_string = resolver
                    .get_full_type(*found, source)
                    .map_or_else(|| INVALID_TYPE.to_string(), |r#type| r#type.to_string());

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found {found_string}, but a function type is required."
                            )),
                    ),
                )
            }
            CompileError::ExpectedArguments {
                function_type,
                found_position: function_position,
                expected_count,
                found_count,
            } => {
                let title = "Incorrect argument count";
                let file_str = source.get_file_as_str(function_position.file_id);
                let function_type_string = resolver
                    .get_full_type(*function_type, source)
                    .map_or_else(|| INVALID_TYPE.to_string(), |r#type| r#type.to_string());

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(function_position.span.as_usize_range())
                            .label(format!(
                                "Expected {expected_count} arguments but found {found_count}."
                            ))
                            .label(format!(
                                "Type {function_type_string} has {expected_count} arguments."
                            )),
                    ),
                )
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InternalError {
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxChild { child_index: u32 },
    MissingSyntaxChildren { start_index: u32, count: u32 },
    MissingDeclaration(DeclarationId),
    MissingLocal(DeclarationId),
    MissingType(TypeId),
    MissingScope(ScopeId),
    MissingSourceFile(SourceFileId),
    MissingSyntaxTree(SourceFileId),
    MissingScopeBinding(SyntaxId),
    MissingDeclarationBinding(SyntaxId),
    MissingTypeBinding(SyntaxId),
    MissingDeclarationType(DeclarationId),
    InvalidSyntaxNode(SyntaxKind),
    InvalidTypeNode(TypeId),
    InvalidDeclarationKind(DeclarationId),
    MissingDeclarationMembers(DeclarationMembers),
    MissingTypeMembers(TypeMembers),
}
