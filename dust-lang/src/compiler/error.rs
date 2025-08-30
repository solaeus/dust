use annotate_snippets::Group;

use crate::{
    Span,
    dust_error::AnnotatedError,
    resolver::{DeclarationId, TypeId},
    syntax_tree::{SyntaxId, SyntaxKind},
};

const INVALID_TREE: &str = "The syntax tree is invalid, this is a bug in the parser.";

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
    MissingSyntaxNode {
        id: SyntaxId,
    },
    MissingType {
        type_id: TypeId,
    },
}

impl AnnotatedError for CompileError {
    fn annotated_error(&self, source: &str) -> Group {
        match self {
            CompileError::InvalidEncodedConstant {
                node_kind,
                position,
                payload,
            } => todo!(),
            CompileError::DivisionByZero {
                node_kind,
                position,
            } => todo!(),
            CompileError::ExpectedItem {
                node_kind,
                position,
            } => todo!(),
            CompileError::ExpectedStatement {
                node_kind,
                position,
            } => todo!(),
            CompileError::ExpectedExpression {
                node_kind,
                position,
            } => todo!(),
            CompileError::MissingChild {
                parent_kind,
                child_index,
            } => todo!(),
            CompileError::MissingConstant { constant_index } => todo!(),
            CompileError::MissingDeclaration { id } => todo!(),
            CompileError::MissingSyntaxNode { id } => todo!(),
            CompileError::MissingType { type_id } => todo!(),
        }
    }
}
