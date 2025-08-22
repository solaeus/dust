use serde::{Deserialize, Serialize};

use crate::{Type, compiler::Scope};

/// A block-local value associated with an identifier.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Local {
    pub identifier_id: u16,

    /// Whether the variable value is allowed to be modified.
    pub is_mutable: bool,

    /// Block scope of the variable, which defines its visibility and lifetime.
    pub scope: Scope,

    /// Value type of the variable.
    pub r#type: Type,
}
