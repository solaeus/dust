//! Scoped variable.

use serde::{Deserialize, Serialize};

use crate::Scope;

/// Scoped variable.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    /// Index of the identifier in the constants list.
    pub identifier_index: u16,

    /// Index of the register where the variable's value is stored.
    pub register_index: u16,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Scope where the variable was declared.
    pub scope: Scope,
}

impl Local {
    /// Creates a new Local instance.
    pub fn new(identifier_index: u16, register_index: u16, is_mutable: bool, scope: Scope) -> Self {
        Self {
            identifier_index,
            register_index,
            is_mutable,
            scope,
        }
    }
}
