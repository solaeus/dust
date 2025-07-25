use serde::{Deserialize, Serialize};

use crate::{Chunk, Type, Value};

use super::Module;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Constant { value: Value, r#type: Type },
    Function(Chunk),
    Module(Module),
}
