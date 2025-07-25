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
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::{self, Display, Formatter};

use crate::{Address, FunctionType, Instruction, Local, Path, Value};
use crate::{OperandType, Span};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Chunk {
    pub(crate) name: Option<Path>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) positions: Vec<Span>,
    pub(crate) constants: Vec<Value>,
    pub(crate) locals: IndexMap<Path, Local>,
    pub(crate) call_arguments: Vec<Vec<(Address, OperandType)>>,

    pub(crate) register_count: usize,
    pub(crate) prototype_index: usize,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.r#type)
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.r#type)
    }
}

impl Eq for Chunk {}

#[cfg(debug_assertions)]
impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.r#type == other.r#type
            && self.instructions == other.instructions
            && self.constants == other.constants
            && self.locals == other.locals
    }
}

#[cfg(not(debug_assertions))]
impl PartialEq for DebugChunk {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.r#type == other.r#type
            && self.instructions == other.instructions
            && self.constants == other.constants
            && self.locals == other.locals
            && self.register_count == other.register_count
            && self.prototype_index == other.prototype_index
    }
}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .as_ref()
            .cmp(&other.name.as_ref())
            .then_with(|| self.r#type.cmp(&other.r#type))
            .then_with(|| self.instructions.cmp(&other.instructions))
            .then_with(|| self.constants.cmp(&other.constants))
            .then_with(|| self.locals.iter().cmp(other.locals.iter()))
            .then_with(|| self.register_count.cmp(&other.register_count))
            .then_with(|| self.prototype_index.cmp(&other.prototype_index))
    }
}
