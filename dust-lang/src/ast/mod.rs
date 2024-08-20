//! In-memory representation of a Dust program.
mod expression;
mod statement;

pub use expression::*;
pub use statement::*;

use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::ContextError;

pub type Span = (usize, usize);

/// In-memory representation of a Dust program.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AbstractSyntaxTree {
    pub statements: VecDeque<Statement>,
}

impl AbstractSyntaxTree {
    pub fn new() -> Self {
        Self {
            statements: VecDeque::new(),
        }
    }

    pub fn with_statements<const LEN: usize>(statements: [Statement; LEN]) -> Self {
        Self {
            statements: statements.into(),
        }
    }
}

impl Default for AbstractSyntaxTree {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Node<T> {
    pub inner: T,
    pub position: Span,
}

impl<T> Node<T> {
    pub fn new(inner: T, position: Span) -> Self {
        Self { inner, position }
    }
}

impl<T: Display> Display for Node<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstError {
    ContextError(ContextError),
    ExpectedType { position: Span },
    ExpectedTupleType { position: Span },
    ExpectedNonEmptyList { position: Span },
    ExpectedRangeableType { position: Span },
}

impl AstError {
    pub fn position(&self) -> Option<Span> {
        match self {
            AstError::ContextError(_) => None,
            AstError::ExpectedType { position } => Some(*position),
            AstError::ExpectedTupleType { position } => Some(*position),
            AstError::ExpectedNonEmptyList { position } => Some(*position),
            AstError::ExpectedRangeableType { position } => Some(*position),
        }
    }
}

impl From<ContextError> for AstError {
    fn from(v: ContextError) -> Self {
        Self::ContextError(v)
    }
}

impl Display for AstError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AstError::ContextError(error) => write!(f, "{}", error),
            AstError::ExpectedType { position } => write!(f, "Expected a type at {:?}", position),
            AstError::ExpectedTupleType { position } => {
                write!(f, "Expected a tuple type at {:?}", position)
            }
            AstError::ExpectedNonEmptyList { position } => {
                write!(f, "Expected a non-empty list at {:?}", position)
            }
            AstError::ExpectedRangeableType { position } => {
                write!(f, "Expected a rangeable type at {:?}", position)
            }
        }
    }
}
