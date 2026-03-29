use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::emitter::JumpId,
    constant_list::ConstantListError,
    error::AnnotatedError,
    instruction::OperandType,
    resolver::{
        Resolver,
        declarations::DeclarationId,
        error::ResolverError,
        symbols::SymbolId,
        types::{Type, TypeId},
    },
    source::{Position, Source, SourceError},
    syntax::{error::SyntaxError, node::SyntaxKind},
};

#[derive(Debug)]
pub enum CompileError {
    CannotApplyOperator {
        operator: SyntaxKind,
        type_id: TypeId,
        operand_position: Position,
    },
    CannotApplyBinaryOperator {
        operator: SyntaxKind,
        operand_position: Position,
        left_type: OperandType,
        left_position: Position,
        right_type: OperandType,
        right_position: Position,
    },
    CannotImport {
        declaration_id: DeclarationId,
        position: Position,
    },
    CannotInferType {
        type_id: TypeId,
        position: Position,
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
    OutOfScopeId {
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
    ExpectedMainFunction,
    Unimplemented {
        syntax_kind: SyntaxKind,
        position: Position,
    },
    ListElementSizeOverflow {
        size: usize,
        type_id: TypeId,
        position: Position,
    },

    ExpectedModuleDeclaration(DeclarationId),
    ExpectedTypeDeclaration(DeclarationId),
    InvalidRegisterCount {
        expected: usize,
        found: usize,
    },
    ExpectedFloatRegister,
    ExpectedIntegerRegister,
    ExpectedLocalDefinition,
    ExpectedEmissionTarget {
        node_kind: SyntaxKind,
    },
    ExpectedJumpPlacement(JumpId),
    Syntax(SyntaxError),
    Resolver(ResolverError),
    ConstantList(ConstantListError),
    Source(SourceError),
    ExpectedSyntaxKind {
        expected: SyntaxKind,
        found: SyntaxKind,
    },
    ExpectedSyntaxKinds {
        expected: &'static [SyntaxKind; 2],
        found: SyntaxKind,
    },
}

impl<'a> AnnotatedError<'a> for CompileError {
    type Context = (&'a Source<'a>, &'a Resolver);

    fn add_report(&self, (source, resolver): Self::Context, groups: &mut Vec<Group<'a>>) {
        match self {
            CompileError::DivisionByZero { position } => {
                let title = "Division by zero".to_string();
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
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
            CompileError::ExpectedIntegerIndex { found, position } => {
                let found_type = resolver
                    .get_external_type(*found, source)
                    .map(|r#type| r#type.to_string())
                    .unwrap_or("<invalid type>".to_string());
                let title = format!("Expected an integer index, found {found_type}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
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
                let title = "Expected a boolean expression".to_string();
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let found_type = match resolver.get_external_type(*found_type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

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
            CompileError::ExpectedFunction {
                node_kind,
                position,
            } => {
                let title = format!("Expected a function, found {node_kind}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
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
            CompileError::OutOfScopeId {
                declaration_id,
                usage_position,
            } => {
                let title = "Undeclared variable".to_string();

                let declaration = match resolver.declarations.get_declaration(*declaration_id) {
                    Ok(declaration) => declaration,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let name = match resolver.symbols.get_symbol(&declaration.symbol_id) {
                    Ok(name) => name,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let file_content = match source.get_file(usage_position.file_id) {
                    Ok(file) => file.content_as_str(),
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
            CompileError::CannotInferType { type_id, position } => {
                let type_node = match resolver.types.get_type(*type_id) {
                    Ok(type_node) => type_node,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let type_declaration_id = match type_node {
                    Type::Algebraic { declaration_id, .. }
                    | Type::FunctionDefinition { declaration_id, .. }
                    | Type::Generic { declaration_id } => Some(*declaration_id),
                    _ => None,
                };
                let type_string = if let Some(declaration_id) = type_declaration_id {
                    let declaration = match resolver.declarations.get_declaration(declaration_id) {
                        Ok(declaration) => declaration,
                        Err(error) => {
                            error.add_report((), groups);

                            return;
                        }
                    };

                    match resolver.symbols.get_symbol(&declaration.symbol_id) {
                        Ok(symbol) => symbol.to_string(),
                        Err(error) => {
                            error.add_report((), groups);

                            return;
                        }
                    }
                } else {
                    match resolver.get_external_type(*type_id, source) {
                        Ok(r#type) => r#type.to_string(),
                        Err(error) => {
                            error.add_report((source, resolver), groups);

                            return;
                        }
                    }
                };
                let title = format!("Cannot infer type {type_string}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title))
                    .elements([Snippet::source(file_content).annotation(
                    AnnotationKind::Primary
                        .span(position.span.as_usize_range())
                        .label(format!(
                            "Type {type_string} was declared here, but its type cannot be inferred."
                        )),
                )]);

                groups.push(group);
            }
            CompileError::TypeConflict {
                expected_type,
                expected_position,
                found_type,
                found_position,
            } => {
                let title = "Type conflict".to_string();
                let expected_type_string = match resolver.get_external_type(*expected_type, source)
                {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

                        return;
                    }
                };
                let found_type_string = match resolver.get_external_type(*found_type, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

                        return;
                    }
                };
                let group = if let Some(expected_position) = expected_position {
                    let expected_file = match source.get_file(expected_position.file_id) {
                        Ok(file) => file,
                        Err(error) => {
                            error.add_report((), groups);

                            return;
                        }
                    };
                    let found_file = match source.get_file(found_position.file_id) {
                        Ok(file) => file,
                        Err(error) => {
                            error.add_report((), groups);

                            return;
                        }
                    };

                    Group::with_title(Level::ERROR.primary_title(title)).elements([
                        Snippet::source(expected_file.content_as_str())
                            .path(found_file.file_name())
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
                    let file = match source.get_file(found_position.file_id) {
                        Ok(file) => file,
                        Err(error) => return error.add_report((), groups),
                    };

                    Group::with_title(Level::ERROR.primary_title(title))
                        .element(
                            Snippet::source(file.content_as_str())
                                .path(file.path_or_name())
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
                let title = "Cannot apply operator".to_string();
                let file_content = match source.get_file(operand_position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let r#type = match resolver.get_external_type(*type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

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
            CompileError::CannotApplyBinaryOperator {
                operator,
                operand_position,
                left_type,
                left_position,
                right_type,
                right_position,
            } => {
                let title = "Cannot apply operator".to_string();
                let file_content = match source.get_file(operand_position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let error_groups = [
                    Group::with_title(Level::ERROR.primary_title(title)).element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(operand_position.span.as_usize_range())
                                .label(format!(
                                    "Cannot apply {operator} to `{left_type}` and `{right_type}`."
                                )),
                        ),
                    ),
                    Group::with_title(Level::ERROR.secondary_title("Left operand type")).element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(left_position.span.as_usize_range())
                                .label(format!("Left operand has type `{left_type}`.")),
                        ),
                    ),
                    Group::with_title(Level::ERROR.secondary_title("Right operand type")).element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Primary
                                .span(right_position.span.as_usize_range())
                                .label(format!("Right operand has type `{right_type}`.")),
                        ),
                    ),
                ];

                groups.extend(error_groups);
            }
            CompileError::CannotIndex { type_id, position } => {
                let r#type = match resolver.get_external_type(*type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

                        return;
                    }
                };
                let title = format!("Cannot index type {type}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
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
                let title = "Undeclared symbol".to_string();
                let file_content = match source.get_file(usage_position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let name_str = match resolver.symbols.get_symbol(symbol_id) {
                    Ok(name) => name,
                    Err(error) => {
                        error.add_report((), groups);

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
                let title = "Unresolved module".to_string();
                let symbol = match resolver.symbols.get_symbol(symbol_id) {
                    Ok(symbol) => symbol,
                    Err(error) => {
                        error.add_report((), groups);

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
            CompileError::ConstantTypeConflict {
                expected,
                found,
                position,
            } => {
                let title = format!("Constant type conflict: expected {expected}, found {found}");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found constant of type {found} here, but expected {expected}"
                            )),
                    ),
                );

                groups.push(group);
            }
            CompileError::CannotMutate { position } => {
                let title = "Cannot mutate immutable value".to_string();
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
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
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };

                let found_type = match resolver.get_external_type(*found, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

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
            CompileError::ExpectedArguments {
                function_type,
                found_position,
                expected_count,
                found_count,
            } => {
                let title = "Incorrect argument count";
                let file_content = match source.get_file(found_position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let function_type = match resolver.get_external_type(*function_type, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

                        return;
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(found_position.span.as_usize_range())
                            .label(format!(
                                "Expected {expected_count} arguments but found {found_count}."
                            ))
                            .label(format!(
                                "Type {function_type} has {expected_count} arguments."
                            )),
                    ),
                );

                groups.push(group);
            }
            CompileError::ExpectedValue {
                node_kind,
                position,
            } => {
                let title = "Expected a value".to_string();
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Expected a value here, but found {node_kind} with type `none`."
                            )),
                    ),
                );

                groups.push(group);
            }
            CompileError::ExpectedNoneType {
                node_kind,
                position,
            } => {
                let title = "Expected type `none`".to_string();
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group =

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Expected type `none` here, but found {node_kind} with a different type."
                            )),
                    ),
                );

                groups.push(group);
            }
            CompileError::CannotInstantiateType { type_id, position } => {
                let title = "Cannot instantiate type".to_string();
                let r#type = match resolver.get_external_type(*type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

                        return;
                    }
                };
                let error_message = format!("Type {type} is an enum and cannot be instantiated.");
                let help_message =
                    "You must specify which variant of the enum you want to crete.".to_string();
                let group = if let Some(position) = position {
                    let file_content = match source.get_file(position.file_id) {
                        Ok(file) => file.content_as_str(),
                        Err(error) => {
                            return error.add_report((), groups);
                        }
                    };

                    Group::with_title(Level::ERROR.primary_title(title))
                        .element(
                            Snippet::source(file_content).annotation(
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
                };

                groups.push(group);
            }
            CompileError::ExpectedNativeFunctionCall { position } => {
                let title = "Expected a native function to be called".to_string();
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
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
                let title = "Expected a main function".to_string();
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Level::HELP.message("A \"main\" function is required to compile the program."),
                );

                groups.push(group);
            }
            CompileError::Unimplemented {
                syntax_kind,
                position,
            } => {
                let title = format!("Unimplemented: `{syntax_kind}`");
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("The use of {syntax_kind} here is not implemented.")),
                    ),
                );

                groups.push(group);
            }
            CompileError::ListElementSizeOverflow {
                size,
                type_id,
                position,
            } => {
                let title = "List element too large".to_string();
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };
                let element_type = match resolver.get_external_type(*type_id, source) {
                    Ok(r#type) => r#type,
                    Err(error) => {
                        error.add_report((source, resolver), groups);

                        return;
                    }
                };
                let found_type_declaration_position = match resolver
                    .declarations
                    .find_type_declaration(*type_id)
                    .map(|found| {
                        found.and_then(|declaration| {
                            declaration.syntax.map(|(position, _)| position)
                        })
                    }) {
                    Ok(found) => found,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };

                let mut group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Type `{element_type}` is {size} bytes, the maximum for list elements is {}.", u16::MAX - 1
                            )),
                    ),
                );

                if let Some(position) = found_type_declaration_position {
                    let file_content = match source.get_file(position.file_id) {
                        Ok(file) => file.content_as_str(),
                        Err(error) => {
                            return error.add_report((), groups);
                        }
                    };

                    group = group.element(
                        Snippet::source(file_content).annotation(
                            AnnotationKind::Context
                                .span(position.span.as_usize_range())
                                .label(format!("Type `{element_type}` was declared here.")),
                        ),
                    );
                }

                groups.push(group);
            }
            CompileError::CannotImport {
                declaration_id,
                position,
            } => {
                let title = "Cannot import".to_string();
                let declaration = match resolver.declarations.get_declaration(*declaration_id) {
                    Ok(declaration) => declaration,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let name = match resolver.symbols.get_symbol(&declaration.symbol_id) {
                    Ok(name) => name,
                    Err(error) => {
                        error.add_report((), groups);

                        return;
                    }
                };
                let file_content = match source.get_file(position.file_id) {
                    Ok(file) => file.content_as_str(),
                    Err(error) => {
                        return error.add_report((), groups);
                    }
                };

                let group = Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(file_content).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Cannot import \"{name}\" here.")),
                    ),
                );

                groups.push(group);
            }
            CompileError::ExpectedModuleDeclaration(_)
            | CompileError::ExpectedTypeDeclaration(_)
            | CompileError::InvalidRegisterCount { .. }
            | CompileError::ExpectedFloatRegister
            | CompileError::ExpectedIntegerRegister
            | CompileError::ExpectedEmissionTarget { .. }
            | CompileError::ExpectedJumpPlacement(_)
            | CompileError::ExpectedSyntaxKind { .. }
            | CompileError::ExpectedSyntaxKinds { .. }
            | CompileError::ExpectedLocalDefinition => {
                self.add_internal_report(groups);
            }
            CompileError::Syntax(error) => error.add_report((), groups),
            CompileError::Resolver(error) => error.add_report((), groups),
            CompileError::ConstantList(error) => error.add_report((), groups),
            CompileError::Source(error) => error.add_report((), groups),
        }
    }
}

impl From<SyntaxError> for CompileError {
    fn from(error: SyntaxError) -> Self {
        CompileError::Syntax(error)
    }
}

impl From<ResolverError> for CompileError {
    fn from(error: ResolverError) -> Self {
        CompileError::Resolver(error)
    }
}

impl From<ConstantListError> for CompileError {
    fn from(error: ConstantListError) -> Self {
        CompileError::ConstantList(error)
    }
}

impl From<SourceError> for CompileError {
    fn from(error: SourceError) -> Self {
        CompileError::Source(error)
    }
}
