//! In-memory representation of a Dust program.
mod expression;
mod statement;

pub use expression::*;
pub use statement::*;

use std::{
    collections::VecDeque,
    fmt::{self, Debug, Display, Formatter},
    num::TryFromIntError,
};

use serde::{Deserialize, Serialize};

use crate::{Context, ContextError};

pub type Span = (usize, usize);

/// In-memory representation of a Dust program.
#[derive(Clone, Serialize, Deserialize)]
pub struct AbstractSyntaxTree {
    pub statements: VecDeque<Statement>,

    #[serde(skip)]
    pub context: Context,
}

impl Debug for AbstractSyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("AbstractSyntaxTree")
            .field("statements", &self.statements)
            .finish()
    }
}

impl Eq for AbstractSyntaxTree {}

impl PartialEq for AbstractSyntaxTree {
    fn eq(&self, other: &Self) -> bool {
        self.statements == other.statements
    }
}

impl PartialOrd for AbstractSyntaxTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AbstractSyntaxTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.statements.cmp(&other.statements)
    }
}

impl AbstractSyntaxTree {
    pub fn new() -> Self {
        Self {
            statements: VecDeque::new(),
            context: Context::new(),
        }
    }

    pub fn with_statements<T: Into<VecDeque<Statement>>>(statements: T) -> Self {
        Self {
            statements: statements.into(),
            context: Context::new(),
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
    ContextError {
        error: ContextError,
        position: Span,
    },
    ExpectedFunctionOrConstructor {
        position: Span,
    },
    ExpectedInteger {
        position: Span,
    },
    ExpectedListType {
        position: Span,
    },
    ExpectedNonEmptyEvaluation {
        position: Span,
    },
    ExpectedNonEmptyList {
        position: Span,
    },
    ExpectedRangeableType {
        position: Span,
    },
    ExpectedStructFieldsType {
        position: Span,
    },
    ExpectedTupleType {
        position: Span,
    },
    FromIntError {
        error: TryFromIntError,
        position: Span,
    },
}

impl AstError {
    pub fn position(&self) -> Span {
        match self {
            AstError::ContextError { position, .. } => *position,
            AstError::ExpectedFunctionOrConstructor { position } => *position,
            AstError::ExpectedInteger { position } => *position,
            AstError::ExpectedListType { position } => *position,
            AstError::ExpectedNonEmptyEvaluation { position } => *position,
            AstError::ExpectedNonEmptyList { position } => *position,
            AstError::ExpectedRangeableType { position } => *position,
            AstError::ExpectedStructFieldsType { position } => *position,
            AstError::ExpectedTupleType { position } => *position,
            AstError::FromIntError { position, .. } => *position,
        }
    }
}

impl Display for AstError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AstError::ContextError { error, position } => {
                write!(f, "Context error at {:?}: {}", position, error)
            }
            AstError::ExpectedFunctionOrConstructor { position } => {
                write!(f, "Expected a function or constructor at {:?}", position)
            }
            AstError::ExpectedInteger { position } => {
                write!(f, "Expected an integer at {:?}", position)
            }
            AstError::ExpectedListType { position } => {
                write!(f, "Expected a type at {:?}", position)
            }
            AstError::ExpectedTupleType { position } => {
                write!(f, "Expected a tuple type at {:?}", position)
            }
            AstError::ExpectedNonEmptyEvaluation { position } => {
                write!(f, "Expected a type at {:?}", position)
            }
            AstError::ExpectedNonEmptyList { position } => {
                write!(f, "Expected a non-empty list at {:?}", position)
            }
            AstError::ExpectedRangeableType { position } => {
                write!(f, "Expected a rangeable type at {:?}", position)
            }
            AstError::ExpectedStructFieldsType { position } => {
                write!(f, "Expected a struct type with fields at {:?}", position)
            }
            AstError::FromIntError { error, position } => {
                write!(f, "Integer conversion error at {:?}: {}", position, error)
            }
        }
    }
}
