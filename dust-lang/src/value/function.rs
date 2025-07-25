use serde::{Deserialize, Serialize};

use crate::FunctionType;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Function {
    pub prototype_index: usize,
    pub r#type: FunctionType,
}
