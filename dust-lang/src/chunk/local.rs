use serde::{Deserialize, Serialize};

use crate::Scope;

/// A scoped variable.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    /// The index of the identifier in the constants table.
    pub identifier_index: u8,

    /// Stack index where the local's value is stored.
    pub register_index: u8,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Scope where the variable was declared.
    pub scope: Scope,
}

impl Local {
    /// Creates a new Local instance.
    pub fn new(identifier_index: u8, register_index: u8, is_mutable: bool, scope: Scope) -> Self {
        Self {
            identifier_index,
            register_index,
            is_mutable,
            scope,
        }
    }
}
