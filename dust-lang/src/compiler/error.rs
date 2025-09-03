use annotate_snippets::{AnnotationKind, Group, Level, Snippet};

use crate::{
    Span,
    dust_error::AnnotatedError,
    resolver::{DeclarationId, TypeId},
    syntax_tree::{SyntaxId, SyntaxKind},
};

const _INVALID_TREE: &str = "The syntax tree is invalid, this is a bug in the parser.";

#[derive(Debug, Clone)]
pub enum CompileError {
    InvalidEncodedConstant {
        node_kind: SyntaxKind,
        position: Span,
        payload: (u32, u32),
    },
    DivisionByZero {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedItem {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedStatement {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedExpression {
        node_kind: SyntaxKind,
        position: Span,
    },
    MissingChild {
        parent_kind: SyntaxKind,
        child_index: u32,
    },
    MissingConstant {
        constant_index: u16,
    },
    MissingDeclaration {
        id: DeclarationId,
    },
    MissingLocalRegister {
        declaration_id: DeclarationId,
    },
    MissingSyntaxNode {
        id: SyntaxId,
    },
    MissingType {
        type_id: TypeId,
    },
}

impl AnnotatedError for CompileError {
    fn annotated_error<'a>(&'a self, source: &'a str) -> Group<'a> {
        match self {
            CompileError::InvalidEncodedConstant {
                node_kind: _,
                position: _,
                payload: _,
            } => todo!(),
            CompileError::DivisionByZero {
                node_kind: _,
                position: _,
            } => todo!(),
            CompileError::ExpectedItem {
                node_kind: _,
                position: _,
            } => todo!(),
            CompileError::ExpectedStatement {
                node_kind: _,
                position: _,
            } => todo!(),
            CompileError::ExpectedExpression {
                node_kind,
                position,
            } => {
                let title = "Expected an expression".to_string();

                Group::with_title(Level::ERROR.primary_title(title)).element(
                    Snippet::source(source).annotation(
                        AnnotationKind::Primary
                            .span(position.as_usize_range())
                            .label(format!("Found {node_kind} but expected an expression here")),
                    ),
                )
            }
            CompileError::MissingChild {
                parent_kind: _,
                child_index: _,
            } => todo!(),
            CompileError::MissingConstant { constant_index: _ } => todo!(),
            CompileError::MissingDeclaration { id: _ } => todo!(),
            CompileError::MissingLocalRegister { declaration_id: _ } => todo!(),
            CompileError::MissingSyntaxNode { id: _ } => todo!(),
            CompileError::MissingType { type_id: _ } => todo!(),
        }
    }
}
