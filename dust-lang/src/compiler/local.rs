use serde::{Deserialize, Serialize};

use crate::{Address, Scope, Type};

/// Block-scoped variable.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Local {
    /// Where the variable's value is stored.
    pub address: Address,

    /// Type of the variable's value.
    pub r#type: Type,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Scope where the variable was declared.
    pub scope: Scope,
}

impl Local {
    /// Creates a new Local instance.
    pub fn new(address: Address, r#type: Type, is_mutable: bool, scope: Scope) -> Self {
        Self {
            address,
            r#type,
            is_mutable,
            scope,
        }
    }
}
