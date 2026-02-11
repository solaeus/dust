use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::{
        Resolver, Symbol, TypeId, TypeNode,
        resolver::{DeclarationId, DeclarationMembers, ScopeId, TypeMembers},
    },
    constant_table::ConstantId,
    dust_error::AnnotatedError,
    instruction::Operation,
    parser::syntax::{SyntaxError, SyntaxId, SyntaxKind},
    source::{Position, Source, SourceFileId},
};

#[derive(Clone, Copy, Debug)]
pub enum CompileError {
    Syntax(SyntaxError),
    Internal(InternalError),

    CannotApplyOperator {
        operator: SyntaxKind,
        type_id: TypeId,
        position: Position,
    },
    CannotInferType {
        type_id: TypeId,
        position: Option<Position>,
    },
    CannotIndex {
        type_id: TypeId,
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
        name: Symbol,
        position: Position,
    },
    UndeclaredType {
        name: Symbol,
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
    ExpectedValue {
        node_kind: SyntaxKind,
        position: Position,
    },
    ExpectedNoneType {
        node_kind: SyntaxKind,
        position: Position,
    },
    CannotInstantiateType {
        type_id: TypeId,
        position: Option<Position>,
    },
    ExpectedNativeFunctionCall {
        position: Position,
    },
}

impl<'a> AnnotatedError<'a> for CompileError {
    type Input = (&'a Source<'a>, &'a Resolver);

    fn annotated_error(&self, (source, resolver): Self::Input) -> Group<'a> {
        match self {
            CompileError::Syntax(syntax_error) => syntax_error.annotated_error(source),
            CompileError::DivisionByZero { position } => {
                let title = "Division by zero".to_string();
                let file_str = source.get_file(position.file_id).content_as_str();

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
                let file_str = source.get_file(position.file_id).content_as_str();

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
                let title = "Expected a boolean expression".to_string();
                let file_str = source.get_file(position.file_id).content_as_str();
                let found_type = match resolver.get_full_type(*found_type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range()).label(format!(
                            "Expected a boolean expression here, but found this {node_kind} with type {found_type}."
                        ))),
                )
            }
            CompileError::ExpectedFunction {
                node_kind,
                position,
            } => {
                let title = format!("Expected a function, found {node_kind}");
                let file_str = source.get_file(position.file_id).content_as_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::UndeclaredVariable { name, position } => {
                let title = "Undeclared variable".to_string();
                let file = source.get_file(position.file_id);
                let file_str = file.content_as_str();
                let name_str = name.get_str(&resolver.constants);

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "\"{name_str}\" was used here, but it was not declared in this scope."
                            )),
                    ),
                )
            }
            CompileError::CannotInferType { type_id, position } => {
                let type_node = match resolver.get_type(*type_id) {
                    Ok(type_node) => type_node,
                    Err(error) => return error.annotated_error((source, resolver)),
                };
                let type_declaration_id = if let TypeNode::Struct { declaration_id, .. }
                | TypeNode::Enum { declaration_id, .. } = type_node
                {
                    Some(*declaration_id)
                } else {
                    None
                };
                let type_string = if let Some(declaration_id) = type_declaration_id {
                    let declaration = match resolver.get_declaration(declaration_id) {
                        Ok(declaration) => declaration,
                        Err(error) => return error.annotated_error((source, resolver)),
                    };

                    declaration.symbol.get_str(&resolver.constants).to_string()
                } else {
                    match resolver.get_full_type(*type_id, source) {
                        Ok(r#type) => r#type.to_string(),
                        Err(error) => return error.annotated_error((source, resolver)),
                    }
                };
                let title = format!("Cannot infer type {type_string}");

                match position {
                    Some(position) => {
                        let file_str = source.get_file(position.file_id).content_as_str();

                        Group::with_title(Level::ERROR.primary_title(title)).elements([
                            Snippet::source(file_str).annotation(
                                AnnotationKind::Primary
                                    .span(position.span.as_usize_range())
                                    .label(format!(
                                        "Type {type_string} was declared here, but its type cannot be inferred."
                                    )),
                            ),
                        ])
                    }
                    None => Group::with_title(Level::ERROR.primary_title(title)).element(
                        Level::ERROR.message(format!("Type {type_string} cannot be inferred.")),
                    ),
                }
            }
            CompileError::TypeConflict {
                expected_type,
                expected_position,
                found_type,
                found_position,
            } => {
                let title = "Type conflict".to_string();

                let expected_type_string = match resolver.get_full_type(*expected_type, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };
                let found_type_string = match resolver.get_full_type(*found_type, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };

                if let Some(expected_position) = expected_position {
                    let expected_file_str =
                        source.get_file(expected_position.file_id).content_as_str();
                    let found_file_str = source.get_file(found_position.file_id).content_as_str();

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
                    let file_str = source.get_file(found_position.file_id).content_as_str();

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
                type_id,
                position,
            } => {
                let r#type = match resolver.get_full_type(*type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };
                let title = format!("Cannot apply operator {operator} to type {type}");
                let file_str = source.get_file(position.file_id).content_as_str();

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
            CompileError::CannotIndex { type_id, position } => {
                let r#type = match resolver.get_full_type(*type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };
                let title = format!("Cannot index type {type}");
                let file_str = source.get_file(position.file_id).content_as_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Attempted to index type {type} here")),
                    ),
                )
            }
            CompileError::UndeclaredType { name, position } => {
                let name_str = name.get_str(&resolver.constants);
                let title = format!("Undeclared type: {name_str}");
                let file_str = source.get_file(position.file_id).content_as_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of undeclared type {name_str} here")),
                    ),
                )
            }
            CompileError::ConstantTypeConflict {
                expected,
                found,
                position,
            } => {
                let title = format!("Constant type conflict: expected {expected}, found {found}");
                let file_str = source.get_file(position.file_id).content_as_str();

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
                let file_str = source.get_file(position.file_id).content_as_str();

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
                let file_str = source.get_file(position.file_id).content_as_str();
                let found_type = match resolver.get_full_type(*found, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found {found_type}, but a function type is required."
                            )),
                    ),
                )
            }
            CompileError::ExpectedArguments {
                function_type,
                found_position,
                expected_count,
                found_count,
            } => {
                let title = "Incorrect argument count";
                let file_str = source.get_file(found_position.file_id).content_as_str();
                let function_type = match resolver.get_full_type(*function_type, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(found_position.span.as_usize_range())
                            .label(format!(
                                "Expected {expected_count} arguments but found {found_count}."
                            ))
                            .label(format!(
                                "Type {function_type} has {expected_count} arguments."
                            )),
                    ),
                )
            }
            CompileError::ExpectedValue {
                node_kind,
                position,
            } => {
                let title = "Expected a value".to_string();
                let file_str = source.get_file(position.file_id).content_as_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Expected a value here, but found {node_kind} with type `none`."
                            )),
                    ),
                )
            }
            CompileError::ExpectedNoneType {
                node_kind,
                position,
            } => {
                let title = "Expected type `none`".to_string();
                let file_str = source.get_file(position.file_id).content_as_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Expected type `none` here, but found {node_kind} with a different type."
                            )),
                    ),
                )
            }
            CompileError::CannotInstantiateType { type_id, position } => {
                let title = "Cannot instantiate type".to_string();
                let r#type = match resolver.get_full_type(*type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => return error.annotated_error((source, resolver)),
                };
                let error_message = format!("Type {type} is an enum and cannot be instantiated.");
                let help_message =
                    "You must specify which variant of the enum you want to crete.".to_string();

                if let Some(position) = position {
                    let file_str = source.get_file(position.file_id).content_as_str();

                    Group::with_title(Level::ERROR.primary_title(title))
                        .element(
                            Snippet::source(file_str).annotation(
                                AnnotationKind::Primary
                                    .span(position.span.as_usize_range())
                                    .label(error_message),
                            ),
                        )
                        .element(Level::HELP.message(help_message))
                } else {
                    Group::with_title(Level::ERROR.primary_title(title)).elements([
                        Level::ERROR.message(error_message),
                        Level::HELP.message(help_message),
                    ])
                }
            }
            CompileError::ExpectedNativeFunctionCall { position } => {
                let title = "Expected a native function to be called".to_string();
                let file_str = source.get_file(position.file_id).content_as_str();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(
                                "Native functions cannot be used as values, they must be called.",
                            ),
                    )
                ).element(Level::HELP.message("To call this native function, add `()` after it."))
                .element(Level::HELP.message("If you wanted to use a function value, declare a function that wraps the native function and use that instead."))
            }
        }
    }
}

impl From<SyntaxError> for CompileError {
    fn from(syntax_error: SyntaxError) -> Self {
        CompileError::Syntax(syntax_error)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InternalError {
    InvalidDeclarationKind(DeclarationId),
    InvalidJumpAnchorInstruction(Operation),
    InvalidNativeFunction(&'static str),
    InvalidSyntaxNode(SyntaxKind),
    InvalidTypeNode(TypeId),
    MissingDeclaration(DeclarationId),
    MissingDeclarationBinding(SyntaxId),
    MissingDeclarationMembers(DeclarationMembers),
    MissingDeclarationPosition(DeclarationId),
    MissingDeclarationType(DeclarationId),
    MissingLocal(DeclarationId),
    MissingScope(ScopeId),
    MissingScopeBinding(SyntaxId),
    MissingSourceFile(SourceFileId),
    MissingSyntaxChild(SyntaxId),
    MissingSyntaxChildren { start_index: u32, count: u32 },
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxTree(SourceFileId),
    MissingType(TypeId),
    MissingTypeBinding(SyntaxId),
    MissingTypeMembers(TypeMembers),
    AnonymousType(DeclarationId),
    MissingDeclarationMember(u32),
    MissingTypeMember(u32),
    MissingConstantString(ConstantId),
}
