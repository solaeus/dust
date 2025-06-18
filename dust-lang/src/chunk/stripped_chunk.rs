use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    Address, DustString, FunctionType, Instruction, Local, OperandType, Span, Value,
    compiler::ChunkCompiler,
};

use super::{Chunk, Disassemble, Disassembler};

#[derive(Clone, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StrippedChunk {
    pub(crate) name: Option<DustString>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,

    pub(crate) constants: Vec<Value<Self>>,
    pub(crate) arguments: Vec<Vec<(Address, OperandType)>>,

    pub(crate) boolean_memory_length: u16,
    pub(crate) byte_memory_length: u16,
    pub(crate) character_memory_length: u16,
    pub(crate) float_memory_length: u16,
    pub(crate) integer_memory_length: u16,
    pub(crate) string_memory_length: u16,
    pub(crate) list_memory_length: u16,
    pub(crate) function_memory_length: u16,
    pub(crate) prototype_index: u16,
}

impl<'a> Chunk<'a> for StrippedChunk {
    fn chunk_type_name() -> &'static str {
        "Stripped Chunk"
    }

    fn name(&self) -> Option<&DustString> {
        self.name.as_ref()
    }

    fn r#type(&self) -> &FunctionType {
        &self.r#type
    }

    fn into_function(self) -> Arc<Self> {
        Arc::new(self)
    }

    fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    fn positions(&self) -> Option<&[Span]> {
        None
    }

    fn constants(&self) -> &[Value<Self>] {
        &self.constants
    }

    fn arguments(&self) -> &[Vec<(Address, OperandType)>] {
        &self.arguments
    }

    fn locals(&self) -> Option<impl Iterator<Item = (&DustString, &Local)>> {
        None::<std::iter::Empty<(&DustString, &Local)>>
    }

    fn boolean_memory_length(&self) -> u16 {
        self.boolean_memory_length
    }

    fn byte_memory_length(&self) -> u16 {
        self.byte_memory_length
    }

    fn character_memory_length(&self) -> u16 {
        self.character_memory_length
    }

    fn float_memory_length(&self) -> u16 {
        self.float_memory_length
    }

    fn integer_memory_length(&self) -> u16 {
        self.integer_memory_length
    }

    fn string_memory_length(&self) -> u16 {
        self.string_memory_length
    }

    fn list_memory_length(&self) -> u16 {
        self.list_memory_length
    }

    fn function_memory_length(&self) -> u16 {
        self.function_memory_length
    }

    fn prototype_index(&self) -> u16 {
        self.prototype_index
    }
}

impl<'a> From<ChunkCompiler<'a, Self>> for StrippedChunk {
    fn from(compiler: ChunkCompiler<'a, Self>) -> Self {
        compiler.finish()
    }
}

impl Disassemble for StrippedChunk {
    fn disassembler<'a, 'w, W: std::io::Write>(
        &'a self,
        writer: &'w mut W,
    ) -> Disassembler<'a, 'w, Self, W> {
        Disassembler::new(self, writer)
    }
}

impl Display for StrippedChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.r#type)
    }
}

impl Debug for StrippedChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.r#type)
    }
}
