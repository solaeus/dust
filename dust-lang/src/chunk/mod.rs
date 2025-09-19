//! Representation of a Dust program or function.
//!
//! A chunk is output by the compiler to represent all the information needed to execute a function.
//! Each function in the source is compiled into its own chunk and stored in the global `prototypes`
//! collection.
// mod disassembler;
mod tui_disassembler;

// pub use disassembler::Disassembler;
pub use tui_disassembler::TuiDisassembler;

use std::fmt::Debug;

use crate::{Address, FunctionType, Instruction, OperandType, Position, Source};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Chunk {
    pub(crate) name_position: Option<Position>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) call_arguments: Vec<(Address, OperandType)>,
    pub(crate) drop_lists: Vec<u16>,

    pub(crate) register_count: u16,
}

impl Chunk {
    pub fn get_name<'a>(&self, source: &'a Source) -> Option<&'a str> {
        let Some(position) = self.name_position else {
            return Some("<anonymous>");
        };
        let file = source.get_file(position.file_id)?;

        file.source_code.get(position.span.as_usize_range())
    }
}
