use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::context::{DeclarationId, ScopeId, TypeId},
    dust_error::AnnotatedError,
    source::{Position, SourceFileId},
    syntax::{SyntaxId, SyntaxKind},
    r#type::Type,
};

#[derive(Debug, Clone)]
pub enum CompileError {
    // User Errors
    CannotImport {
        name: String,
        position: Position,
    },
    CannotInferListType {
        position: Position,
    },
    DivisionByZero {
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
    ExpectedBooleanExpression {
        node_kind: SyntaxKind,
        position: Position,
    },
    ExpectedFunctionType {
        type_id: TypeId,
    },
    ExpectedList {
        found_type: Type,
        position: Position,
    },
    UndeclaredVariable {
        name: String,
        position: Position,
    },
    CannotMutate {
        name: String,
        position: Position,
    },
    TypeMismatch {
        expected: Type,
        expected_position: Position,
        found: Type,
        found_position: Position,
    },

    // Internal Errors (from incorrect Parser output)
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
    ExpectedFunctionBody {
        node_kind: SyntaxKind,
        position: Position,
    },
    MissingChild {
        parent_kind: SyntaxKind,
        child_index: u32,
    },
    MissingChildren {
        parent_kind: SyntaxKind,
        start_index: u32,
        count: u32,
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
        syntax_id: SyntaxId,
    },
    MissingType {
        type_id: TypeId,
    },
    MissingScope {
        scope_id: ScopeId,
    },
    MissingPrototype {
        declaration_id: DeclarationId,
    },
    MissingSourceFile {
        file_id: SourceFileId,
    },
    MissingSyntaxTree {
        file_id: SourceFileId,
    },
    MissingPayloads {
        payload_start: u32,
        payload_count: u32,
    },
    MissingPosition {
        declaration_id: DeclarationId,
    },
    UnresolvedFunctionType {
        function_type: TypeId,
    },
    MissingDeclarations {
        name: String,
    },
    MissingNativeFunction {
        native_function: String,
    },
    MissingScopeBinding {
        syntax_id: SyntaxId,
    },
    MissingTypeMembers {
        start_index: u32,
        count: u32,
    },
}

impl AnnotatedError for CompileError {
    fn file_id(&self) -> SourceFileId {
        match self {
            CompileError::ChildIndexOutOfBounds { .. } => SourceFileId::default(),
            CompileError::InvalidEncodedConstant { position, .. } => position.file_id,
            CompileError::InvalidNativeFunction { position, .. } => position.file_id,
            CompileError::DivisionByZero { position, .. } => position.file_id,
            CompileError::DuplicateFunctionDeclaration {
                second_position, ..
            } => second_position.file_id,
            CompileError::ExpectedItem { position, .. } => position.file_id,
            CompileError::ExpectedStatement { position, .. } => position.file_id,
            CompileError::ExpectedBooleanExpression { position, .. } => position.file_id,
            CompileError::ExpectedExpression { position, .. } => position.file_id,
            CompileError::ExpectedFunction { position, .. } => position.file_id,
            CompileError::ExpectedFunctionBody { position, .. } => position.file_id,
            CompileError::ExpectedFunctionType { .. } => SourceFileId::default(),
            CompileError::ExpectedList { position, .. } => position.file_id,
            CompileError::MissingChild { .. } => SourceFileId::default(),
            CompileError::MissingChildren { .. } => SourceFileId::default(),
            CompileError::MissingConstant { .. } => SourceFileId::default(),
            CompileError::MissingDeclaration { .. } => SourceFileId::default(),
            CompileError::MissingDeclarations { .. } => SourceFileId::default(),
            CompileError::MissingLocal { .. } => SourceFileId::default(),
            CompileError::MissingSyntaxNode { .. } => SourceFileId::default(),
            CompileError::MissingType { .. } => SourceFileId::default(),
            CompileError::MissingTypeMembers { .. } => SourceFileId::default(),
            CompileError::MissingScope { .. } => SourceFileId::default(),
            CompileError::MissingPrototype { .. } => SourceFileId::default(),
            CompileError::MissingSourceFile { file_id } => *file_id,
            CompileError::MissingSyntaxTree { .. } => SourceFileId::default(),
            CompileError::MissingPayloads { .. } => SourceFileId::default(),
            CompileError::MissingPosition { .. } => SourceFileId::default(),
            CompileError::UndeclaredVariable { position, .. } => position.file_id,
            CompileError::UnresolvedFunctionType { .. } => SourceFileId::default(),
            CompileError::CannotImport { position, .. } => position.file_id,
            CompileError::CannotInferListType { position } => position.file_id,
            CompileError::MissingNativeFunction { .. } => SourceFileId::default(),
            CompileError::CannotMutate { position, .. } => position.file_id,
            CompileError::MissingScopeBinding { .. } => SourceFileId::default(),
            CompileError::TypeMismatch {
                expected_position, ..
            } => expected_position.file_id,
        }
    }

    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a> {
        match self {
            CompileError::CannotMutate { name, position } => {
                let title = format!("Cannot mutate immutable variable: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!(
                                "Attempted mutation of immutable variable {name} here"
                            )),
                    ),
                )
            }
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
            CompileError::DivisionByZero { position } => {
                let title = "Division by zero".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
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
            CompileError::ExpectedBooleanExpression {
                node_kind,
                position,
            } => {
                let title = format!("Expected a boolean expression, found {node_kind}");

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
            CompileError::ExpectedFunctionType { type_id } => {
                let title = format!("Expected a function type, found {type_id:?}");

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::ExpectedList {
                found_type,
                position,
            } => {
                let title = format!("Expected a list type, found {found_type}");

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
            CompileError::MissingChildren {
                parent_kind,
                start_index,
                count,
            } => {
                let title = format!(
                    "Expected {count} children starting at {start_index} on {parent_kind}, but they were missing, this is a bug in the parser or compiler"
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
            CompileError::MissingDeclarations { name } => {
                let title = format!(
                    "Declarations with name {name} were missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingLocal { declaration_id } => {
                let title = format!(
                    "Local for declaration id {declaration_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSyntaxNode { syntax_id: id } => {
                let title = format!(
                    "Syntax node with id {id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingType { type_id } => {
                let title = format!(
                    "Type node with id {type_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingScope { scope_id: id } => {
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
            CompileError::MissingSourceFile { file_id } => {
                let title = format!(
                    "Source file with id {file_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSyntaxTree { file_id } => {
                let title = format!(
                    "Syntax tree for file id {file_id:?} was missing, this is a bug in the parser or compiler"
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
            CompileError::UndeclaredVariable { name, position } => {
                let title = format!("Undeclared variable: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of undeclared variable {name} here")),
                    ),
                )
            }
            CompileError::UnresolvedFunctionType { function_type } => {
                let title = format!(
                    "Function type {function_type:?} could not be resolved, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::CannotImport { name, position } => {
                let title =
                    format!("Cannot import: {name}, only modules and functions can be imported");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Cannot import {name}")),
                    ),
                )
            }
            CompileError::CannotInferListType { position } => {
                let title = "Cannot infer list type, please provide an explicit type".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::MissingNativeFunction { native_function } => {
                let title = format!(
                    "Native function {native_function} was missing, this is a bug in the parser or
                    compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingScopeBinding { syntax_id } => {
                let title = format!(
                    "Scope binding for syntax id {syntax_id:?} was missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingTypeMembers { start_index, count } => {
                let title = format!(
                    "Expected {count} type members starting at {start_index}, but they were missing, this is a bug in the parser or compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::TypeMismatch {
                expected,
                expected_position,
                found,
                found_position,
            } => {
                let title = format!("Type mismatch: expected {expected}, found {found}");

                Group::with_title(Level::ERROR.primary_title(title)).elements(vec![
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(found_position.span.as_usize_range())
                            .label(format!("Found type {found} here")),
                    ),
                    Snippet::source(source).annotation(
                        AnnotationKind::Context
                            .span(expected_position.span.as_usize_range())
                            .label(format!("Expected type {expected} here")),
                    ),
                ])
            }
        }
    }
}
