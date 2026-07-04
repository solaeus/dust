use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::{
        context::{
            Context,
            declarations::DeclarationId,
            symbols::SymbolId,
            types::{Type, TypeId},
        },
        prototype_emitter::JumpId,
    },
    constants::{ConstantsError, value::ConstantValue},
    dust_type::DustType,
    error::DustError,
    source::{Position, Source, SourceCodeId, SourceError, Span},
    syntax::{Syntax, SyntaxId, error::SyntaxError, node::SyntaxKind},
};

#[derive(Clone, Debug)]
pub enum CompileError {
    // User errors
    CannotApplyOperator {
        operator: SyntaxKind,
        type_id: TypeId,
        operand_position: Position,
    },
    CannotInferType {
        type_id: TypeId,
    },
    CannotIndex {
        type_id: TypeId,
        position: Position,
    },
    CannotMutate {
        position: Position,
    },
    ConstantValueOverflow {
        position: Position,
    },
    ConstantBinaryOverflow {
        left_value: ConstantValue,
        left_span: Span,
        right_value: ConstantValue,
        right_span: Span,
        operator: SyntaxKind,
        source_id: SourceCodeId,
    },
    ConstantUnaryOverflow {
        value: ConstantValue,
        operand_span: Span,
        operator: SyntaxKind,
        source_id: SourceCodeId,
    },
    InvalidConstantExponent {
        base_value: ConstantValue,
        base_span: Span,
        exponent_value: ConstantValue,
        exponent_span: Span,
        operator: SyntaxKind,
        source_id: SourceCodeId,
    },
    ExpectedIntegerIndex {
        found: TypeId,
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
    DeclarationOutOfScope {
        declaration_id: DeclarationId,
        usage_position: Position,
    },
    Undeclared {
        symbol_id: SymbolId,
        usage_position: Position,
    },
    UnresolvedModule {
        symbol_id: SymbolId,
    },
    ExpectedFunctionType {
        found: TypeId,
        position: Position,
    },
    ExpectedValue {
        source_id: SourceCodeId,
        syntax_id: SyntaxId,
    },
    IndexOutOfBounds {
        index: usize,
        length: usize,
        position: Position,
    },
    ExpectedNativeFunctionCall {
        position: Position,
    },
    ExpectedMainFunction,
    ExpectedIndexableType {
        type_id: TypeId,
    },
    SelfTypeOutsideOfImpl {
        position: Position,
    },

    // Internal errors
    Source(SourceError),
    Syntax(SyntaxError),
    ConstantList(ConstantsError),
    ExpectedTypeDeclaration(DeclarationId),
    InvalidRegisterAllocation,
    ExpectedJumpPlacement(JumpId),
    MissingDeclarationBinding(SyntaxId),
    MissingTypeMember(u32),
    MissingTypeBinding(SyntaxId),
    ExpectedFunctionDefinition(DeclarationId),
    ExpectedAlgebraicTypeDefinition(DeclarationId),
    MissingTypeArgument(DeclarationId),
    ExpectedConcreteType,
    ExpectedFieldDefinition(DeclarationId),
    ExpectedAllocation,
    InvalidTypeBinding(TypeId),
    ExpectedConstantDefinition(DeclarationId),
    ExpectedArrayType(TypeId),
    InvalidEmission,
    UnexpectedSyntax {
        expected: &'static [SyntaxKind],
        found: SyntaxKind,
    },
    ExpectedLocalDefinition(DeclarationId),
    ExpectedVariantDefinition(DeclarationId),
    ExpectedStructDefinition(DeclarationId),
    ScopeStackUnderflow,
    ExpectedSyntax {
        expected: &'static [SyntaxKind],
    },
    ExpectedAlgebraicType(TypeId),
    ExpectedEncodedValue {
        found: ConstantValue,
    },
    InvalidContext,
    ExpectedFunctionDefinitionType(TypeId),
    ExpectedInferredType(TypeId),
    UnexpectedType(TypeId),
}

impl From<SyntaxError> for CompileError {
    fn from(error: SyntaxError) -> Self {
        CompileError::Syntax(error)
    }
}

impl From<ConstantsError> for CompileError {
    fn from(error: ConstantsError) -> Self {
        CompileError::ConstantList(error)
    }
}

impl From<SourceError> for CompileError {
    fn from(error: SourceError) -> Self {
        CompileError::Source(error)
    }
}

impl<'a> DustError<'a> for CompileError {
    type Info = (&'a Source<'a>, &'a Syntax, &'a Context);

    fn add_report(&self, (source, syntax, context): Self::Info, groups: &mut Vec<Group<'a>>) {
        match self {
            CompileError::ExpectedIntegerIndex { found, position } => {
                let found_type = context
                    .get_external_type(*found)
                    .map(|r#type| r#type.to_string())
                    .unwrap_or("<invalid type>".to_string());
                let title = format!("Expected an integer index, found {found_type}");
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                );

                groups.push(group);
            }
            CompileError::ExpectedBooleanExpression {
                found: found_type_id,
                node_kind,
                position,
            } => {
                let title = "Expected a boolean expression";
                let file_content = match source.get_content(*position) {
                    Ok(file) => file,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let found_type = match context.get_external_type(*found_type_id) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range()).label(format!(
                            "Expected a boolean expression here, but found this {node_kind} with type {found_type}."
                        ))),
                );

                groups.push(group);
            }
            CompileError::DeclarationOutOfScope {
                declaration_id,
                usage_position,
            } => {
                let title = "Declaration out of scope";

                let declaration = context.declarations.get_declaration(*declaration_id);
                let name = match context.symbols.get_symbol(&declaration.symbol_id) {
                    Ok(name) => name,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let file_content = match source.get_content(*usage_position) {
                    Ok(content) => content,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };

                let group = if let Some((position, _)) = declaration.syntax {
                    Group::with_title(Level::ERROR.primary_title(title)).element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(format!(
                                    "\"{name}\" was declared here, but it is not in scope at the usage site."
                                )),
                        ),
                    )
                } else {
                    Group::with_title(Level::ERROR.primary_title(title)).element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(usage_position.span.as_usize_range())
                                .label(format!("Attempted to use \"{name}\" but it is not available in this scope.")),
                        ),
                    )
                    .element(Level::HELP.message(
                        "\"{name}\" is not in the source code. It was declared externally via the API.",
                    ))
                };

                groups.push(group);
            }
            CompileError::CannotInferType { type_id } => {
                let r#type = context.types.get_type(*type_id);
                let declaration = match r#type {
                    Type::Algebraic { declaration_id, .. }
                    | Type::FunctionDefinition { declaration_id, .. }
                    | Type::Generic { declaration_id } => {
                        Some(context.declarations.get_declaration(*declaration_id))
                    }
                    _ => None,
                };
                let type_string = if let Some(declaration) = declaration {
                    match context.symbols.get_symbol(&declaration.symbol_id) {
                        Ok(symbol) => Some(symbol.to_string()),
                        Err(error) => {
                            error.add_report((source, syntax, context), groups);

                            return;
                        }
                    }
                } else {
                    None
                };
                let title = "Cannot infer type";
                let message = if let Some(type_string) = type_string {
                    format!("Cannot infer type `{type_string}`.")
                } else {
                    "Cannot infer this type.".to_string()
                };
                let group = if let Some((position, _)) =
                    declaration.and_then(|declaration| declaration.syntax)
                {
                    let file_content = match source.get_content(position) {
                        Ok(content) => content,
                        Err(error) => {
                            error.add_report((), groups);

                            return;
                        }
                    };

                    Group::with_title(Level::ERROR.primary_title(title)).element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(position.span.as_usize_range())
                                .label(message),
                        ),
                    )
                } else {
                    Group::with_title(Level::ERROR.primary_title(title))
                        .element(Level::ERROR.message(message))
                };

                groups.push(group);
            }
            CompileError::TypeConflict {
                expected_type,
                expected_position,
                found_type,
                found_position,
            } => {
                let title = "Type conflict";
                let expected_type_string = match context.get_external_type(*expected_type) {
                    Ok(r#type) => match r#type {
                        DustType::Struct(struct_type) => struct_type.name,
                        _ => r#type.to_string(),
                    },
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let found_type_string = match context.get_external_type(*found_type) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let found_file = source.get_code(found_position.source_id);
                let group = if let Some(expected_position) = expected_position {
                    let expected_file = source.get_code(expected_position.source_id);

                    Group::with_title(Level::ERROR.primary_title(title)).elements([
                        Snippet::source(expected_file.content_as_str())
                            .path(expected_file.path_or_name())
                            .annotation(
                                AnnotationKind::Context
                                    .span(expected_position.span.as_usize_range())
                                    .label(format!(
                                        "Type `{expected_type_string}` was established here."
                                    )),
                            ),
                        Snippet::source(found_file.content_as_str()).annotation(
                            AnnotationKind::Primary
                                .span(found_position.span.as_usize_range())
                                .label(format!("Found `{found_type_string}` here.")),
                        ),
                    ])
                } else {
                    Group::with_title(Level::ERROR.primary_title(title))
                        .element(
                            Snippet::source(found_file.content_as_str())
                                .path(found_file.path_or_name())
                                .fold(false)
                                .annotation(
                                    AnnotationKind::Primary
                                        .span(found_position.span.as_usize_range())
                                        .label(format!("Found `{found_type_string}` here.")),
                                ),
                        )
                        .element(Level::ERROR.message(format!(
                            "Expected this expression to have type `{expected_type_string}`."
                        )))
                };

                groups.push(group);
            }
            CompileError::CannotApplyOperator {
                operator,
                type_id,
                operand_position,
            } => {
                let title = "Cannot apply operator";
                let file_content = match source.get_content(*operand_position) {
                    Ok(content) => content,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let r#type = match context.get_external_type(*type_id) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(operand_position.span.as_usize_range())
                            .label(format!("Cannot apply {operator} to `{type}`.")),
                    ),
                );

                groups.push(group);
            }
            CompileError::CannotIndex { type_id, position } => {
                let r#type = match context.get_external_type(*type_id) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let title = format!("Cannot index type {type}");
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Attempted to index type {type} here")),
                    ),
                );

                groups.push(group);
            }
            CompileError::Undeclared {
                symbol_id,
                usage_position,
            } => {
                let title = "Undeclared symbol";
                let file_content = source.get_code(usage_position.source_id).content_as_str();
                let name_str = match context.symbols.get_symbol(symbol_id) {
                    Ok(name) => name,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(usage_position.span.as_usize_range())
                            .label(format!("\"{name_str}\" was never declared.")),
                    ),
                );

                groups.push(group);
            }
            CompileError::UnresolvedModule { symbol_id } => {
                let title = "Unresolved module";
                let symbol = match context.symbols.get_symbol(symbol_id) {
                    Ok(symbol) => symbol,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Level::ERROR.message(format!(
                        "Could not find \"{symbol}.ds\" or \"{symbol}/mod.ds\"."
                    )),
                );

                groups.push(group);
            }
            CompileError::CannotMutate { position } => {
                let title = "Cannot mutate immutable value";
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                );

                groups.push(group);
            }
            CompileError::ExpectedFunctionType { found, position } => {
                let title = "Expected a function type";
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };

                let found_type = match context.get_external_type(*found) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found {found_type}, but a function type is required."
                            )),
                    ),
                );

                groups.push(group);
            }
            CompileError::ExpectedValue {
                source_id,
                syntax_id,
            } => {
                let syntax = match syntax
                    .get_tree(*source_id)
                    .and_then(|tree| tree.read_node(*syntax_id))
                {
                    Ok(syntax) => syntax,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };

                let title = "Expected a value";
                let file_content = source.get_code(*source_id).content_as_str();

                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(syntax.node.span.as_usize_range())
                            .label(format!(
                                "Expected a value here, but found {} with type `()`.",
                                syntax.node.kind
                            )),
                    ),
                );

                groups.push(group);
            }
            CompileError::IndexOutOfBounds {
                index,
                length,
                position,
            } => {
                let title = "Index out of bounds";
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Index {index} is out of bounds for array of length {length}."
                            )),
                    ),
                );

                groups.push(group);
            }
            CompileError::ExpectedNativeFunctionCall { position } => {
                let title = "Expected a native function to be called";
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };

                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(
                                "Native functions cannot be used as values, they must be called.",
                            ),
                    )
                )
                .element(Level::HELP.message("To call this native function, add `()` after it."))
                .element(Level::HELP.message("If you wanted to use a function value, declare a function that wraps the native function and use that instead."));

                groups.push(group);
            }
            CompileError::ExpectedMainFunction => {
                let title = "Expected a main function";
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Level::HELP.message("A \"main\" function is required to compile the program."),
                );

                groups.push(group);
            }
            CompileError::ConstantBinaryOverflow {
                left_value,
                left_span,
                right_value,
                right_span,
                operator,
                source_id,
            } => {
                let title = "Constant overflow";
                let file_content = source.get_code(*source_id).content_as_str();

                let group = Group::with_title(Level::ERROR.primary_title(title))
                    .element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(left_span.as_usize_range())
                                .label(format!("Left operand has value {left_value}.")),
                        ),
                    )
                    .element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(right_span.as_usize_range())
                                .label(format!("Right operand has value {right_value}.")),
                        ),
                    )
                    .element(Level::ERROR.message(format!(
                        "Applying operator {operator} to these values causes an overflow."
                    )));

                groups.push(group);
            }
            CompileError::ConstantUnaryOverflow {
                value,
                operand_span,
                operator,
                source_id,
            } => {
                let title = "Constant overflow";
                let file_content = source.get_code(*source_id).content_as_str();

                let group = Group::with_title(Level::ERROR.primary_title(title))
                    .element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(operand_span.as_usize_range())
                                .label(format!("Operand has value {value}.")),
                        ),
                    )
                    .element(Level::ERROR.message(format!(
                        "Applying operator {operator} to this value causes an overflow."
                    )));

                groups.push(group);
            }
            CompileError::InvalidConstantExponent {
                base_value,
                base_span,
                exponent_value,
                exponent_span,
                operator,
                source_id,
            } => {
                let title = "Invalid constant exponent";
                let file_content = source.get_code(*source_id).content_as_str();

                let group = Group::with_title(Level::ERROR.primary_title(title))
                    .element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(base_span.as_usize_range())
                                .label(format!("Base operand has value {base_value}.")),
                        ),
                    )
                    .element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(exponent_span.as_usize_range())
                                .label(format!("Exponent operand has value {exponent_value}.")),
                        ),
                    )
                    .element(Level::ERROR.message(format!(
                        "Applying operator {operator} to these values is invalid."
                    )));

                groups.push(group);
            }
            CompileError::ExpectedIndexableType { type_id } => {
                let title = "Expected an indexable type";
                let r#type = match context.get_external_type(*type_id) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, syntax, context), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::ERROR.message(format!("Type {type} cannot be indexed.")));

                groups.push(group);
            }
            CompileError::UnexpectedSyntax { expected, found } => {
                let title = "Unexpected syntax";
                let expected_string = expected
                    .iter()
                    .map(|kind| format!("`{kind}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let found_string = format!("`{found}`");
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Level::ERROR.message(format!(
                        "Expected one of the following syntax kinds: {expected_string}, but found {found_string}."
                    )),
                );

                groups.push(group);
            }
            CompileError::SelfTypeOutsideOfImpl { position } => {
                let title = "Use of `Self` type outside of impl";
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label("The `Self` type can only be used within impl blocks and trait definitions."),
                    ),
                );

                groups.push(group);
            }
            CompileError::ConstantValueOverflow { position } => {
                let title = "Constant value overflow";
                let file_content = match source.get_content(*position) {
                    Ok(content) => content,
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label("This constant value exceeds the maximum allowed for its type."),
                    ),
                );

                groups.push(group);
            }
            CompileError::Syntax(error) => error.add_report((), groups),
            CompileError::ConstantList(error) => error.add_report((), groups),
            CompileError::Source(error) => error.add_report((), groups),
            CompileError::ExpectedTypeDeclaration(_)
            | CompileError::InvalidRegisterAllocation
            | CompileError::ExpectedJumpPlacement(_)
            | CompileError::ExpectedFieldDefinition { .. }
            | CompileError::ExpectedAllocation
            | CompileError::MissingDeclarationBinding(_)
            | CompileError::MissingTypeMember(_)
            | CompileError::MissingTypeBinding(_)
            | CompileError::ExpectedFunctionDefinition(_)
            | CompileError::ExpectedAlgebraicTypeDefinition(_)
            | CompileError::MissingTypeArgument(_)
            | CompileError::ExpectedConcreteType
            | CompileError::InvalidTypeBinding(_)
            | CompileError::ExpectedConstantDefinition(_)
            | CompileError::ExpectedArrayType(_)
            | CompileError::InvalidEmission
            | CompileError::ExpectedLocalDefinition(_)
            | CompileError::ExpectedVariantDefinition(_)
            | CompileError::ExpectedStructDefinition(_)
            | CompileError::ExpectedSyntax { .. }
            | CompileError::ScopeStackUnderflow
            | CompileError::ExpectedAlgebraicType(_)
            | CompileError::ExpectedEncodedValue { .. }
            | CompileError::InvalidContext
            | CompileError::ExpectedFunctionDefinitionType(_)
            | CompileError::ExpectedInferredType(_)
            | CompileError::UnexpectedType(_) => {
                self.add_internal_report(groups);
            }
        }
    }
}
