use serde::{Deserialize, Serialize};

use crate::{Chunk, ConcreteValue};

use super::Module;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Constant(ConcreteValue),
    Function(Box<Chunk>),
    Module(Module),
}
