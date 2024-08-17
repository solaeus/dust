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
