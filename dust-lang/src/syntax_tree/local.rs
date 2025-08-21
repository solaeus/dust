use serde::{Deserialize, Serialize};

use crate::{Span, Type, syntax_tree::Scope};

/// A block-local value associated with an identifier.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Local {
    /// The source code position of the variable's identifier.
    pub identifier_position: Span,

    /// Whether the variable value is allowed to be modified.
    pub is_mutable: bool,

    /// Block scope of the variable, which defines its visibility and lifetime.
    pub scope: Scope,

    /// Value type of the variable.
    pub r#type: Type,
}
