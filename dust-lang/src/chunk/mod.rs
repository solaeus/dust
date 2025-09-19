//! Representation of a Dust program or function.
//!
//! A chunk is output by the compiler to represent all the information needed to execute a Dust
//! program. In addition to the program itself, each function in the source is compiled into its own
//! chunk and stored in the `prototypes` field of its parent. Thus, a chunk can also represent a
//! function prototype.
//!
//! Chunks have a name when they belong to a named function. They also have a type, so the input
//! parameters and the type of the return value are statically known.
// mod disassembler;
mod tui_disassembler;

// pub use disassembler::Disassembler;
pub use tui_disassembler::TuiDisassembler;

use std::fmt::Debug;

use crate::{
    Address, CompileError, Instruction, OperandType, Resolver, Source,
    resolver::{DeclarationId, TypeId},
};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Chunk {
    pub(crate) declaration_id: DeclarationId,
    pub(crate) type_id: TypeId,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) call_arguments: Vec<(Address, OperandType)>,
    pub(crate) drop_lists: Vec<u16>,

    pub(crate) register_count: u16,
    pub(crate) is_recursive: bool,
}

impl Chunk {
    pub fn get_name<'a>(
        &self,
        resolver: &'a Resolver,
        source: &'a Source,
    ) -> Result<&'a str, CompileError> {
        match self.declaration_id {
            DeclarationId::MAIN => Ok("main"),
            DeclarationId::ANONYMOUS => Ok("anonymous"),
            id => {
                let declaration = resolver
                    .get_declaration(id)
                    .ok_or(CompileError::MissingDeclaration { declaration_id: id })?;
                let file_source = source
                    .get_file(declaration.identifier_position.file_index)
                    .ok_or(CompileError::MissingSourceFile {
                        file_index: declaration.identifier_position.file_index,
                    })?;
                let name =
                    &file_source.source_code[declaration.identifier_position.span.as_usize_range()];

                Ok(name)
            }
        }
    }
}
