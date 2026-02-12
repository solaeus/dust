use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::{
        Resolver, Symbol, TypeId, TypeNode,
        resolver::{DeclarationId, DeclarationMembers, ScopeId, TypeMembers},
    },
    constant_table::ConstantKey,
    dust_error::AnnotatedError,
    instruction::Operation,
    parser::syntax::{SyntaxError, SyntaxId, SyntaxKind},
    prototype::ReadOnlyError,
    source::{Position, Source, SourceFileId},
};

#[derive(Clone, Copy, Debug)]
pub enum CompileError {
    Syntax(SyntaxError),
    Internal(InternalCompileError),

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
    OutOfScope {
        declaration_id: DeclarationId,
        usage_position: Position,
    },
    Undeclared {
        symbol: Symbol,
        usage_position: Position,
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
            CompileError::Internal(internal_error) => {
                let title = format!("Internal compiler error: {internal_error}");

                Group::with_title(Level::ERROR.primary_title(title))
            }
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
            CompileError::OutOfScope {
                declaration_id,
                usage_position,
            } => {
                let title = "Undeclared variable".to_string();

                let declaration = match resolver.get_declaration(*declaration_id) {
                    Ok(declaration) => declaration,
                    Err(error) => return error.annotated_error((source, resolver)),
                };
                let name = match resolver.get_symbol_name(&declaration.symbol) {
                    Ok(name) => name,
                    Err(error) => return error.annotated_error((source, resolver)),
                };
                let file = source.get_file(usage_position.file_id);
                let file_str = file.content_as_str();

                let Some(position) = declaration.position else {
                    return Group::with_title(Level::ERROR.primary_title(title)).element(
                        Snippet::source(file_str).annotation(
                            AnnotationKind::Primary
                                .span(usage_position.span.as_usize_range())
                                .label(format!("Attempted to use \"{name}\" but it is not available in this scope.")),
                        ),
                    )
                    .element(Level::HELP.message(
                        "\"{name}\" is not in the source code. It was declared externally via the API.",
                    ))
                    ;
                };

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "\"{name}\" was used here, but it was not declared in this scope."
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

                    resolver
                        .get_symbol_name(&declaration.symbol)
                        .expect("Types cannot be anonymous")
                        .to_string()
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
            CompileError::Undeclared {
                symbol,
                usage_position,
            } => {
                let title = "Undeclared symbol".to_string();
                let file_str = source.get_file(usage_position.file_id).content_as_str();
                let name_str = match resolver.get_symbol_name(symbol) {
                    Ok(name) => name,
                    Err(error) => return error.annotated_error((source, resolver)),
                };

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_str).annotation(
                        AnnotationKind::Primary
                            .span(usage_position.span.as_usize_range())
                            .label(format!("\"{name_str}\" was never declared.")),
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
pub enum InternalCompileError {
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
    MissingConstantString(ConstantKey),
    AnonymousSymbolLookup,
    PrototypeList(ReadOnlyError),
}

impl Display for InternalCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalCompileError::InvalidDeclarationKind(declaration_id) => {
                write!(
                    f,
                    "Invalid declaration kind for declaration ID {}",
                    declaration_id.inner()
                )
            }
            InternalCompileError::InvalidJumpAnchorInstruction(instruction) => {
                write!(f, "Invalid jump anchor instruction: {:?}", instruction)
            }
            InternalCompileError::InvalidNativeFunction(name) => {
                write!(f, "Invalid native function: {}", name)
            }
            InternalCompileError::InvalidSyntaxNode(kind) => {
                write!(f, "Invalid syntax node kind: {:?}", kind)
            }
            InternalCompileError::InvalidTypeNode(type_id) => {
                write!(f, "Invalid type node for type ID {}", type_id.inner())
            }
            InternalCompileError::MissingDeclaration(declaration_id) => {
                write!(
                    f,
                    "Missing declaration for declaration ID {}",
                    declaration_id.inner()
                )
            }
            InternalCompileError::MissingDeclarationBinding(syntax_id) => {
                write!(
                    f,
                    "Missing declaration binding for syntax ID {}",
                    syntax_id.inner()
                )
            }
            InternalCompileError::MissingDeclarationMembers(members) => {
                write!(f, "Missing declaration members: {:?}", members)
            }
            InternalCompileError::MissingDeclarationPosition(declaration_id) => {
                write!(
                    f,
                    "Missing declaration position for declaration ID {}",
                    declaration_id.inner()
                )
            }
            InternalCompileError::MissingDeclarationType(declaration_id) => {
                write!(
                    f,
                    "Missing declaration type for declaration ID {}",
                    declaration_id.inner()
                )
            }
            InternalCompileError::MissingLocal(declaration_id) => {
                write!(
                    f,
                    "Missing local for declaration ID {}",
                    declaration_id.inner()
                )
            }
            InternalCompileError::MissingScope(scope_id) => {
                write!(f, "Missing scope for scope ID {}", scope_id.inner())
            }
            InternalCompileError::MissingScopeBinding(syntax_id) => {
                write!(
                    f,
                    "Missing scope binding for syntax ID {}",
                    syntax_id.inner()
                )
            }
            InternalCompileError::MissingSourceFile(file_id) => {
                write!(f, "Missing source file for file ID {}", file_id.inner())
            }
            InternalCompileError::MissingSyntaxChild(syntax_id) => {
                write!(
                    f,
                    "Missing syntax child for syntax ID {}",
                    syntax_id.inner()
                )
            }
            InternalCompileError::MissingSyntaxChildren { start_index, count } => {
                write!(
                    f,
                    "Missing {} syntax children starting from index {}",
                    count, start_index
                )
            }
            InternalCompileError::MissingSyntaxNode(syntax_id) => {
                write!(f, "Missing syntax node for syntax ID {}", syntax_id.inner())
            }
            InternalCompileError::MissingSyntaxTree(file_id) => {
                write!(f, "Missing syntax tree for file ID {}", file_id.inner())
            }
            InternalCompileError::MissingType(type_id) => {
                write!(f, "Missing type for type ID {}", type_id.inner())
            }
            InternalCompileError::MissingTypeBinding(syntax_id) => {
                write!(
                    f,
                    "Missing type binding for syntax ID {}",
                    syntax_id.inner()
                )
            }
            InternalCompileError::MissingTypeMembers(members) => {
                write!(f, "Missing type members: {:?}", members)
            }
            InternalCompileError::AnonymousType(declaration_id) => {
                write!(
                    f,
                    "Anonymous type for declaration ID {}",
                    declaration_id.inner()
                )
            }
            InternalCompileError::MissingDeclarationMember(index) => {
                write!(f, "Missing declaration member at index {}", index)
            }
            InternalCompileError::MissingTypeMember(index) => {
                write!(f, "Missing type member at index {}", index)
            }
            InternalCompileError::MissingConstantString(constant_key) => {
                write!(
                    f,
                    "Missing constant string for constant ID {constant_key:?}",
                )
            }
            InternalCompileError::AnonymousSymbolLookup => {
                write!(f, "Attempted to lookup an anonymous symbol")
            }
            InternalCompileError::PrototypeList(error) => {
                write!(f, "{error}")
            }
        }
    }
}
