//! Representation of a Dust program or function.
//!
//! A chunk is output by the compiler to represent all the information needed to execute a Dust
//! program. In addition to the program itself, each function in the source is compiled into its own
//! chunk and stored in the `prototypes` field of its parent. Thus, a chunk can also represent a
//! function prototype.
//!
//! Chunks have a name when they belong to a named function. They also have a type, so the input
//! parameters and the type of the return value are statically known.
mod disassembler;

pub use disassembler::Disassembler;
use serde::{Deserialize, Serialize};

use std::fmt::{Debug, Display};

use crate::{Address, FunctionType, Instruction, Local, OperandType, Path, Value};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Chunk {
    pub(crate) name: Option<Path>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) constants: Vec<Value>,
    pub(crate) locals: Vec<(Path, Local)>,
    pub(crate) call_argument_lists: Vec<Vec<(Address, OperandType)>>,

    pub(crate) register_count: u16,
    pub(crate) drop_lists: Vec<Vec<u16>>,
    pub(crate) prototype_index: u16,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.r#type)
    }
}
