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
mod tui_disassembler;

pub use disassembler::Disassembler;
pub use tui_disassembler::TuiDisassembler;

use std::fmt::{Debug, Display};

use crate::{Address, ConstantTable, FunctionType, Instruction, OperandType};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Chunk {
    pub(crate) name: Option<String>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) constants: ConstantTable,
    pub(crate) call_arguments: Vec<(Address, OperandType)>,
    pub(crate) drop_lists: Vec<u16>,

    pub(crate) register_count: u16,
    pub(crate) prototype_index: u16,
    pub(crate) is_recursive: bool,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.r#type)
    }
}
