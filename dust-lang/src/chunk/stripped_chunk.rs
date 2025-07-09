use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{FunctionType, Instruction, Local, Path, Value, compiler::CompiledData};

use super::{Chunk, Disassemble, Disassembler};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StrippedChunk {
    pub(crate) r#type: FunctionType,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) constants: Vec<Value<Self>>,
    pub(crate) register_count: usize,
    pub(crate) prototype_index: usize,
}

impl Chunk for StrippedChunk {
    fn new(data: CompiledData<Self>) -> Self {
        StrippedChunk {
            r#type: data.r#type,
            instructions: data.instructions,
            constants: data.constants,
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

    #[inline(always)]
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    #[inline(always)]
    fn constants(&self) -> &[Value<Self>] {
        &self.constants
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

impl Eq for StrippedChunk {}

#[cfg(debug_assertions)]
impl PartialEq for StrippedChunk {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type
            && self.instructions == other.instructions
            && self.constants == other.constants
    }
}

#[cfg(not(debug_assertions))]
impl PartialEq for StrippedChunk {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type
            && self.instructions == other.instructions
            && self.constants == other.constants
            && self.register_count == other.register_count
            && self.prototype_index == other.prototype_index
    }
}

impl PartialOrd for StrippedChunk {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrippedChunk {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.r#type
            .cmp(&other.r#type)
            .then_with(|| self.instructions.cmp(&other.instructions))
            .then_with(|| self.constants.cmp(&other.constants))
            .then_with(|| self.register_count.cmp(&other.register_count))
            .then_with(|| self.prototype_index.cmp(&other.prototype_index))
    }
}
