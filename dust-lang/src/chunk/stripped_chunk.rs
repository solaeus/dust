use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    Address, FunctionType, Instruction, Local, OperandType, Path, Span, Value,
    compiler::CompiledData,
};

use super::{Chunk, Disassemble, Disassembler};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StrippedChunk {
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,

    pub(crate) constants: Vec<Value<Self>>,
    pub(crate) arguments: Vec<Vec<(Address, OperandType)>>,
    pub(super) parameters: Vec<Address>,

    pub(crate) register_count: usize,
    pub(crate) prototype_index: usize,
}

impl Chunk for StrippedChunk {
    fn new(data: CompiledData<Self>) -> Self {
        StrippedChunk {
            r#type: data.r#type,
            instructions: data.instructions,
            constants: data.constants,
            arguments: data.arguments,
            parameters: data.parameters,
            register_count: data.register_count,
            prototype_index: data.prototype_index,
        }
    }

    fn chunk_type_name() -> &'static str {
        "Stripped Chunk"
    }

    fn name(&self) -> Option<&Path> {
        None
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

    fn parameters(&self) -> &[Address] {
        &self.parameters
    }

    fn locals(&self) -> Option<impl Iterator<Item = (&Path, &Local)>> {
        None::<std::iter::Empty<(&Path, &Local)>>
    }

    fn register_count(&self) -> usize {
        self.register_count
    }

    fn prototype_index(&self) -> usize {
        self.prototype_index
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
        write!(f, "stripped_chunk",)
    }
}
