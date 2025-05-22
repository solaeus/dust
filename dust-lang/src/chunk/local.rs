//! Scoped variable.

use serde::{Deserialize, Serialize};

use crate::{Address, Scope, Type};

/// Scoped variable.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    /// Index of the identifier in the string constants list.
    pub identifier_index: u16,

    /// Pointer to where the variable's value is stored.
    pub address: Address,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Type of the variable's value.
    pub r#type: Type,

    /// Scope where the variable was declared.
    pub scope: Scope,
}

impl Local {
    /// Creates a new Local instance.
    pub fn new(
        identifier_index: u16,
        address: Address,
        r#type: Type,
        is_mutable: bool,
        scope: Scope,
    ) -> Self {
        Self {
            identifier_index,
            address,
            r#type,
            is_mutable,
            scope,
        }
    }
}
