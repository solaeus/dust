use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    Address, Disassembler, DustString, FunctionType, Instruction, Local, Span, chunk::Disassemble,
    compiler::ChunkCompiler, r#type::TypeKind,
};

use super::{Chunk, StrippedChunk};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct FullChunk {
    pub(crate) name: Option<DustString>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) positions: Vec<Span>,

    pub(crate) character_constants: Vec<char>,
    pub(crate) float_constants: Vec<f64>,
    pub(crate) integer_constants: Vec<i64>,
    pub(crate) string_constants: Vec<DustString>,
    pub(crate) prototypes: Vec<Arc<FullChunk>>,
    pub(crate) arguments: Vec<Vec<(Address, TypeKind)>>,

    pub(crate) locals: IndexMap<DustString, Local>,

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

impl FullChunk {
    pub fn strip(self) -> StrippedChunk {
        StrippedChunk::from(self)
    }
}

impl Chunk for FullChunk {
    fn from_chunk_compiler<'a>(compiler: ChunkCompiler<'a, Self>) -> Self {
        compiler.finish()
    }

    fn chunk_type_name() -> &'static str {
        "Full Chunk"
    }

    fn name(&self) -> Option<&DustString> {
        self.name.as_ref()
    }

    fn r#type(&self) -> &FunctionType {
        &self.r#type
    }

    fn as_function(self) -> Arc<Self> {
        Arc::new(self)
    }

    fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    fn positions(&self) -> Option<&[Span]> {
        Some(&self.positions)
    }

    fn character_constants(&self) -> &[char] {
        &self.character_constants
    }

    fn float_constants(&self) -> &[f64] {
        &self.float_constants
    }

    fn integer_constants(&self) -> &[i64] {
        &self.integer_constants
    }

    fn string_constants(&self) -> &[DustString] {
        &self.string_constants
    }

    fn prototypes(&self) -> &[Arc<Self>] {
        &self.prototypes
    }

    fn arguments(&self) -> &[Vec<(Address, TypeKind)>] {
        &self.arguments
    }

    fn locals(&self) -> Option<impl Iterator<Item = (&DustString, &Local)>> {
        Some(self.locals.iter())
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

impl Disassemble for FullChunk {
    fn disassembler<'a, W: std::io::Write>(
        &'a self,
        writer: &'a mut W,
    ) -> Disassembler<'a, Self, W> {
        Disassembler::new(self, writer)
    }
}

impl Display for FullChunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.r#type)
    }
}

impl Debug for FullChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.r#type)
    }
}

impl Eq for FullChunk {}

/// For testing purposes, ignore the "memory_length" fields so that we don't have to write them them
/// when writing Chunks for tests.
#[cfg(debug_assertions)]
impl PartialEq for FullChunk {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.r#type == other.r#type
            && self.instructions == other.instructions
            && self.positions == other.positions
            && self.character_constants == other.character_constants
            && self.float_constants == other.float_constants
            && self.integer_constants == other.integer_constants
            && self.string_constants == other.string_constants
            && self.locals == other.locals
            && self.prototypes == other.prototypes
            && self.arguments == other.arguments
            && self.prototype_index == other.prototype_index
    }
}

#[cfg(not(debug_assertions))]
impl PartialEq for FullChunk {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.r#type == other.r#type
            && self.instructions == other.instructions
            && self.positions == other.positions
            && self.character_constants == other.character_constants
            && self.float_constants == other.float_constants
            && self.integer_constants == other.integer_constants
            && self.string_constants == other.string_constants
            && self.locals == other.locals
            && self.prototypes == other.prototypes
            && self.arguments == other.arguments
            && self.boolean_memory_length == other.boolean_memory_length
            && self.byte_memory_length == other.byte_memory_length
            && self.character_memory_length == other.character_memory_length
            && self.float_memory_length == other.float_memory_length
            && self.integer_memory_length == other.integer_memory_length
            && self.string_memory_length == other.string_memory_length
            && self.list_memory_length == other.list_memory_length
            && self.function_memory_length == other.function_memory_length
            && self.prototype_index == other.prototype_index
    }
}
