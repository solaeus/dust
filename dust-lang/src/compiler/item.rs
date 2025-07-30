use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{Chunk, Module, Type, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Constant { value: Value, r#type: Type },
    Function(Arc<Chunk>),
    Module(Module),
}
