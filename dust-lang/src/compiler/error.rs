use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    compiler::{
        TypeId,
        resolver::{DeclarationId, ScopeId},
    },
    dust_error::AnnotatedError,
    source::{Position, SourceFileId},
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
    DivisionByZero {
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
    ExpectedFunctionType {
        type_id: TypeId,
    },
    TypeConflict {
        expected: Type,
        found: Type,
        position: Position,
    },
    UndeclaredVariable {
        name: String,
        position: Position,
    },
    UndeclaredType {
        name: String,
        position: Position,
    },

    // Internal Errors (from incorrect Parser output)
    InvalidNativeFunction {
        name: String,
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
    MissingSourceFile {
        file_id: SourceFileId,
    },
    MissingSyntaxTree {
        file_id: SourceFileId,
    },
    MissingScopeBinding {
        syntax_id: SyntaxId,
    },
    MissingTypeMembers {
        start_index: u32,
        count: u32,
    },
    InvalidSyntaxNode {
        kind: SyntaxKind,
    },
    MissingDeclarationBinding {
        syntax_id: SyntaxId,
    },
    MissingTypeBinding {
        syntax_id: SyntaxId,
    },
    MissingDeclarationType {
        declaration_id: DeclarationId,
    },
    MissingDeclarationMember {
        declaration_id: DeclarationId,
    },
}

impl AnnotatedError for CompileError {
    fn file_id(&self) -> SourceFileId {
        match self {
            CompileError::InvalidNativeFunction { position, .. } => position.file_id,
            CompileError::DivisionByZero { position } => position.file_id,
            CompileError::ExpectedItem { position, .. } => position.file_id,
            CompileError::ExpectedStatement { position, .. } => position.file_id,
            CompileError::ExpectedBooleanExpression { position, .. } => position.file_id,
            CompileError::ExpectedExpression { position, .. } => position.file_id,
            CompileError::ExpectedFunction { position, .. } => position.file_id,
            CompileError::ExpectedFunctionType { .. } => SourceFileId::default(),
            CompileError::MissingChild { .. } => SourceFileId::default(),
            CompileError::MissingChildren { .. } => SourceFileId::default(),
            CompileError::MissingDeclaration { .. } => SourceFileId::default(),
            CompileError::MissingLocal { .. } => SourceFileId::default(),
            CompileError::MissingSyntaxNode { .. } => SourceFileId::default(),
            CompileError::MissingType { .. } => SourceFileId::default(),
            CompileError::MissingTypeMembers { .. } => SourceFileId::default(),
            CompileError::MissingScope { .. } => SourceFileId::default(),
            CompileError::MissingSourceFile { file_id } => *file_id,
            CompileError::MissingSyntaxTree { .. } => SourceFileId::default(),
            CompileError::UndeclaredVariable { position, .. } => position.file_id,
            CompileError::CannotInferType { position } => position.file_id,
            CompileError::MissingScopeBinding { .. } => SourceFileId::default(),
            CompileError::TypeConflict { position, .. } => position.file_id,
            CompileError::InvalidSyntaxNode { .. } => SourceFileId::default(),
            CompileError::MissingDeclarationBinding { .. } => SourceFileId::default(),
            CompileError::MissingTypeBinding { .. } => SourceFileId::default(),
            CompileError::CannotApplyOperator { position, .. } => position.file_id,
            CompileError::CannotIndex { position, .. } => position.file_id,
            CompileError::MissingDeclarationType { .. } => SourceFileId::default(),
            CompileError::UndeclaredType { position, .. } => position.file_id,
            CompileError::AmbiguousType { position, .. } => position.file_id,
            CompileError::MissingDeclarationMember { .. } => SourceFileId::default(),
        }
    }

    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a> {
        match self {
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
            CompileError::ExpectedFunctionType { type_id } => {
                let title = format!("Expected a function type, found {type_id:?}");

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingChild {
                parent_kind,
                child_index,
            } => {
                let title = format!(
                    "Expected child {child_index} on {parent_kind}, but it was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingChildren {
                parent_kind,
                start_index,
                count,
            } => {
                let title = format!(
                    "Expected {count} children starting at {start_index} on {parent_kind}, but they were missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingDeclaration { declaration_id: id } => {
                let title = format!(
                    "Declaration with id {id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingLocal { declaration_id } => {
                let title = format!(
                    "Local for declaration id {declaration_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSyntaxNode { syntax_id: id } => {
                let title = format!(
                    "Syntax node with id {id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingType { type_id } => {
                let title = format!(
                    "Type node with id {type_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingScope { scope_id: id } => {
                let title =
                    format!("Scope with id {id:?} was missing, this is a bug in the compiler");

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSourceFile { file_id } => {
                let title = format!(
                    "Source file with id {file_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingSyntaxTree { file_id } => {
                let title = format!(
                    "Syntax tree for file id {file_id:?} was missing, this is a bug in the compiler"
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
            CompileError::CannotInferType { position } => {
                let title = "Cannot infer type, please provide an explicit type".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source)
                        .annotation(AnnotationKind::Primary.span(position.span.as_usize_range())),
                )
            }
            CompileError::MissingScopeBinding { syntax_id } => {
                let title = format!(
                    "Scope binding for syntax id {syntax_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingTypeMembers { start_index, count } => {
                let title = format!(
                    "Expected {count} type members starting at {start_index}, but they were missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::TypeConflict {
                expected,
                found,
                position,
            } => {
                let title = format!("Type mismatch: expected {expected}, found {found}");

                Group::with_title(Level::ERROR.primary_title(title)).elements(vec![
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Found type {found} here")),
                    ),
                ])
            }
            CompileError::InvalidSyntaxNode { kind } => {
                let title = format!(
                    "Invalid syntax node: {kind} is in an invalid position, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingDeclarationBinding { syntax_id } => {
                let title = format!(
                    "Declaration binding for syntax id {syntax_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::MissingTypeBinding { syntax_id } => {
                let title = format!(
                    "Type binding for syntax id {syntax_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::CannotApplyOperator {
                operator,
                r#type,
                position,
            } => {
                let title = format!("Cannot apply operator {operator} to type {type}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
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
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Attempted to index type {type} here")),
                    ),
                )
            }
            CompileError::MissingDeclarationType { declaration_id } => {
                let title = format!(
                    "Type for declaration id {declaration_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            CompileError::UndeclaredType { name, position } => {
                let title = format!("Undeclared type: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of undeclared type {name} here")),
                    ),
                )
            }
            CompileError::AmbiguousType { name, position } => {
                let title = format!("Ambiguous type: {name}");

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.span.as_usize_range())
                            .label(format!("Use of ambiguous type {name} here")),
                    ),
                )
            }
            CompileError::MissingDeclarationMember { declaration_id } => {
                let title = format!(
                    "Member for declaration id {declaration_id:?} was missing, this is a bug in the compiler"
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
