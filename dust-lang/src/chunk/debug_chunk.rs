use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    Address, Disassembler, FunctionType, Instruction, Local, OperandType, Path, Value,
    chunk::Disassemble, compiler::CompiledData,
};

use super::Chunk;

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DebugChunk {
    pub(crate) name: Option<Path>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) constants: Vec<Value>,
    pub(crate) locals: IndexMap<Path, Local>,
    pub(crate) call_arguments: Vec<Vec<(Address, OperandType)>>,

    pub(crate) register_count: usize,
    pub(crate) prototype_index: usize,
}

impl Chunk for DebugChunk {
    fn new(data: CompiledData) -> Self {
        DebugChunk {
            name: data.name,
            r#type: data.r#type,
            instructions: data.instructions,
            constants: data.constants,
            locals: data.locals,
            call_arguments: data.call_arguments,
            register_count: data.register_count,
            prototype_index: data.prototype_index,
        }
    }

    fn chunk_type_name() -> &'static str {
        "Full Chunk"
    }

    fn name(&self) -> Option<&Path> {
        self.name.as_ref()
    }

    fn r#type(&self) -> &FunctionType {
        &self.r#type
    }

    fn into_function(self) -> Arc<Self> {
        Arc::new(self)
    }

    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    fn constants(&self) -> &[Value] {
        &self.constants
    }

    fn locals(&self) -> Option<impl Iterator<Item = (&Path, &Local)>> {
        Some(self.locals.iter())
    }

    fn call_arguments(&self) -> &Vec<Vec<(Address, OperandType)>> {
        &self.call_arguments
    }

    fn register_count(&self) -> usize {
        self.register_count
    }

    fn prototype_index(&self) -> usize {
        self.prototype_index
    }
}

impl Disassemble for DebugChunk {
    fn disassembler<'a, 'w, W: std::io::Write>(
        &'a self,
        writer: &'w mut W,
    ) -> Disassembler<'a, 'w, Self, W> {
        Disassembler::new(self, writer)
    }
}

impl Display for DebugChunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.r#type)
    }
}

impl Debug for DebugChunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut buffer = Vec::new();
        let _ = self.disassembler(&mut buffer).disassemble();
        let string = String::from_utf8_lossy(&buffer);

        write!(f, "\n{string}")
    }
}

impl Eq for DebugChunk {}

/// For testing purposes, ignore the "memory_length" fields so that we don't have to write them them
/// when writing Chunks for tests.
#[cfg(debug_assertions)]
impl PartialEq for DebugChunk {
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

impl PartialOrd for DebugChunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DebugChunk {
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
