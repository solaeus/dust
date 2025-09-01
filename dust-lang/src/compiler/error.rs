use annotate_snippets::Group;

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
    MissingSyntaxNode {
        id: SyntaxId,
    },
    MissingType {
        type_id: TypeId,
    },
}

impl AnnotatedError for CompileError {
    fn annotated_error<'a>(&'a self, _source: &str) -> Group<'a> {
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
                node_kind: _,
                position: _,
            } => todo!(),
            CompileError::MissingChild {
                parent_kind: _,
                child_index: _,
            } => todo!(),
            CompileError::MissingConstant { constant_index: _ } => todo!(),
            CompileError::MissingDeclaration { id: _ } => todo!(),
            CompileError::MissingSyntaxNode { id: _ } => todo!(),
            CompileError::MissingType { type_id: _ } => todo!(),
        }
    }
}
