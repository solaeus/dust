use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    Position,
    dust_error::AnnotatedError,
    resolver::{DeclarationId, ScopeId, TypeId},
    syntax_tree::{SyntaxId, SyntaxKind},
};

#[derive(Debug, Clone)]
pub enum CompileError {
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
    MissingFunctionPrototype {
        declaration_id: DeclarationId,
    },
    MissingSourceFile {
        file_index: u32,
    },
}

impl AnnotatedError for CompileError {
    fn file_index(&self) -> usize {
        match self {
            CompileError::InvalidEncodedConstant { position, .. } => position.file_index as usize,
            CompileError::InvalidNativeFunction { position, .. } => position.file_index as usize,
            CompileError::DivisionByZero { position, .. } => position.file_index as usize,
            CompileError::ExpectedItem { position, .. } => position.file_index as usize,
            CompileError::ExpectedStatement { position, .. } => position.file_index as usize,
            CompileError::ExpectedExpression { position, .. } => position.file_index as usize,
            CompileError::ExpectedFunction { position, .. } => position.file_index as usize,
            CompileError::ExpectedFunctionBody { position, .. } => position.file_index as usize,
            CompileError::MissingChild { .. } => 0,
            CompileError::MissingConstant { .. } => 0,
            CompileError::MissingDeclaration { .. } => 0,
            CompileError::MissingLocal { .. } => 0,
            CompileError::MissingSyntaxNode { .. } => 0,
            CompileError::MissingType { .. } => 0,
            CompileError::MissingScope { .. } => 0,
            CompileError::MissingFunctionPrototype { .. } => 0,
            CompileError::MissingSourceFile { file_index } => *file_index as usize,
        }
    }

    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a> {
        match self {
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
            CompileError::MissingFunctionPrototype { declaration_id } => {
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
        }
    }
}
