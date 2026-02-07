use crate::{
    source::Position,
    syntax::{SyntaxId, SyntaxKind, SyntaxPayload},
};

#[derive(Debug)]
pub enum SyntaxError {
    ExpectedItem {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedStatement {
        found: SyntaxKind,
        position: Position,
    },
    ExpectedExpression {
        found: SyntaxKind,
        position: Position,
    },
    Internal(InternalSyntaxError),
}

#[derive(Debug)]
pub enum InternalSyntaxError {
    ExpectedChild,
    MissingSyntaxNode(SyntaxId),
    MissingSyntaxChildren(SyntaxPayload),
}
