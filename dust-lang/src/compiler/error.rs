use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    Position,
    dust_error::AnnotatedError,
    resolver::{DeclarationId, ScopeId, TypeId, TypeNode},
    syntax_tree::{SyntaxId, SyntaxKind},
};

#[derive(Debug, Clone)]
pub enum CompileError {
    ChildIndexOutOfBounds {
        parent_kind: SyntaxKind,
        children_start: u32,
        child_count: u32,
    },
    InvalidEncodedConstant {
        node_kind: SyntaxKind,
        position: Position,
        payload: (u32, u32),
    },
    InvalidNativeFunction {
        name: String,
        position: Position,
    },
    DivisionByZero {
        node_kind: SyntaxKind,
        position: Position,
    },
    DuplicateFunctionDeclaration {
        identifier: String,
        first_position: Position,
        second_position: Position,
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
    ExpectedFunctionBody {
        node_kind: SyntaxKind,
        position: Position,
    },
    ExpectedFunctionType {
        type_node: TypeNode,
        position: Position,
    },
    MissingChild {
        parent_kind: SyntaxKind,
        child_index: u32,
    },
    MissingConstant {
        constant_index: u16,
    },
    MissingDeclaration {
        declaration_id: DeclarationId,
    },
    MissingLocal {
        declaration_id: DeclarationId,
    },
    MissingSyntaxNode {
        id: SyntaxId,
    },
    MissingType {
        type_id: TypeId,
    },
    MissingScope {
        id: ScopeId,
    },
    MissingPrototype {
        declaration_id: DeclarationId,
    },
    MissingSourceFile {
        file_index: u32,
    },
    MissingSyntaxTree {
        declaration_id: DeclarationId,
    },
    MissingPayloads {
        payload_start: u32,
        payload_count: u32,
    },
    MissingPosition {
        declaration_id: DeclarationId,
    },
}

impl AnnotatedError for CompileError {
    fn file_index(&self) -> u32 {
        match self {
            CompileError::ChildIndexOutOfBounds { .. } => 0,
            CompileError::InvalidEncodedConstant { position, .. } => position.file_index,
            CompileError::InvalidNativeFunction { position, .. } => position.file_index,
            CompileError::DivisionByZero { position, .. } => position.file_index,
            CompileError::DuplicateFunctionDeclaration {
                second_position, ..
            } => second_position.file_index,
            CompileError::ExpectedItem { position, .. } => position.file_index,
            CompileError::ExpectedStatement { position, .. } => position.file_index,
            CompileError::ExpectedExpression { position, .. } => position.file_index,
            CompileError::ExpectedFunction { position, .. } => position.file_index,
            CompileError::ExpectedFunctionBody { position, .. } => position.file_index,
            CompileError::ExpectedFunctionType { position, .. } => position.file_index,
            CompileError::MissingChild { .. } => 0,
            CompileError::MissingConstant { .. } => 0,
            CompileError::MissingDeclaration { .. } => 0,
            CompileError::MissingLocal { .. } => 0,
            CompileError::MissingSyntaxNode { .. } => 0,
            CompileError::MissingType { .. } => 0,
            CompileError::MissingScope { .. } => 0,
            CompileError::MissingPrototype { .. } => 0,
            CompileError::MissingSourceFile { file_index } => *file_index,
            CompileError::MissingSyntaxTree { .. } => 0,
            CompileError::MissingPayloads { .. } => 0,
            CompileError::MissingPosition { .. } => 0,
        }
    }

    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a> {
        match self {
            CompileError::ChildIndexOutOfBounds {
                parent_kind,
                children_start,
                child_count,
            } => {
                let title = format!(
                    "Child index out of bounds on {parent_kind}: has {child_count} children starting at index {children_start}"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::InvalidEncodedConstant {
                node_kind,
                position,
                payload,
            } => {
                let title = "Invalid encoded constant".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Found {node_kind} with invalid encoded constant {payload:?} here"
                            )),
                    ),
                )
            }
            CompileError::InvalidNativeFunction { name, position } => {
                let title = format!("Invalid native function: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Found invalid native function {name} here")),
                    ),
                )
            }
            CompileError::DivisionByZero {
                node_kind,
                position,
            } => {
                let title = "Division by zero".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Found {node_kind} that divides by zero here")),
                    ),
                )
            }
            CompileError::DuplicateFunctionDeclaration {
                identifier,
                first_position,
                second_position,
            } => {
                let title = format!("Duplicate function declaration: {identifier}");

                Group::with_title(Level::ERROR.primary_title(title)).elements(vec![
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(second_position.span.as_usize_range())
                            .label("Duplicate declaration found here".to_string()),
                    ),
                    Snippet::source(source).annotation(
                        AnnotationKind::Context
                            .span(first_position.span.as_usize_range())
                            .label("First declaration found here".to_string()),
                    ),
                ])
            }
            CompileError::ExpectedItem {
                node_kind,
                position,
            } => {
                let title = format!("Expected an item, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedStatement {
                node_kind,
                position,
            } => {
                let title = format!("Expected a statement, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedExpression {
                node_kind,
                position,
            } => {
                let title = format!("Expected an expression, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedFunction {
                node_kind,
                position,
            } => {
                let title = format!("Expected a function, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedFunctionBody {
                node_kind,
                position,
            } => {
                let title = format!("Expected a function body, found {node_kind}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::ExpectedFunctionType {
                type_node: type_id,
                position,
            } => {
                let title = format!("Expected a function type, found {type_id:?}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::MissingChild {
                parent_kind,
                child_index,
            } => {
                let title = format!(
                    "Expected child {child_index} on {parent_kind}, but it was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingConstant { constant_index: _ } => {
                let title =
                    "A constant was missing, this is a bug in the parser or compiler".to_string();

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingDeclaration { declaration_id: id } => {
                let title = format!(
                    "Declaration with id {id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingLocal { declaration_id } => {
                let title = format!(
                    "Local for declaration id {declaration_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSyntaxNode { id } => {
                let title = format!(
                    "Syntax node with id {id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingType { type_id } => {
                let title = format!(
                    "Type with id {type_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingScope { id } => {
                let title = format!(
                    "Scope with id {id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingPrototype { declaration_id } => {
                let title = format!(
                    "Function prototype for declaration id {declaration_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSourceFile { file_index } => {
                let title = format!(
                    "Source file with index {file_index} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSyntaxTree { declaration_id } => {
                let title = format!(
                    "Syntax tree for declaration id {declaration_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingPayloads {
                payload_start,
                payload_count,
            } => {
                let title = format!(
                    "Expected {payload_count} payloads starting at {payload_start}, but they were missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingPosition { declaration_id } => {
                let title = format!(
                    "Position for declaration id {declaration_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
