use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    Address, Disassembler, DustString, FunctionType, Instruction, List, Local, Span, Value,
    chunk::Disassemble, compiler::ChunkCompiler, instruction::OperandType,
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

    pub(crate) constants: Vec<Value<Self>>,
    pub(crate) arguments: Vec<Vec<(Address, OperandType)>>,

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
        StrippedChunk {
            name: self.name,
            r#type: self.r#type,
            instructions: self.instructions,
            constants: self
                .constants
                .into_iter()
                .map(|value| match value {
                    Value::Boolean(boolean) => Value::boolean(boolean),
                    Value::Byte(byte) => Value::byte(byte),
                    Value::Character(character) => Value::character(character),
                    Value::Float(float) => Value::float(float),
                    Value::Integer(integer) => Value::integer(integer),
                    Value::String(string) => Value::string(string),
                    Value::List(list) => match list {
                        List::Boolean(booleans) => Value::boolean_list(booleans),
                        List::Byte(bytes) => Value::byte_list(bytes),
                        List::Character(characters) => Value::character_list(characters),
                        List::Float(floats) => Value::float_list(floats),
                        List::Integer(integers) => Value::integer_list(integers),
                        List::String(strings) => Value::string_list(strings),
                        List::List(lists) => {
                            let stripped_lists = lists
                                .into_iter()
                                .map(|list| list.strip())
                                .collect::<Vec<_>>();

                            Value::list_list(stripped_lists)
                        }
                        List::Function(prototypes) => {
                            let stripped_prototypes = prototypes
                                .into_iter()
                                .map(|prototype| {
                                    let prototype = Arc::unwrap_or_clone(prototype);

                                    Arc::new(prototype.strip())
                                })
                                .collect::<Vec<_>>();

                            Value::function_list(stripped_prototypes)
                        }
                    },
                    Value::Function(prototype) => {
                        let prototype = Arc::unwrap_or_clone(prototype);

                        Value::function(prototype.strip())
                    }
                })
                .collect(),
            arguments: self.arguments,
            boolean_memory_length: self.boolean_memory_length,
            byte_memory_length: self.byte_memory_length,
            character_memory_length: self.character_memory_length,
            float_memory_length: self.float_memory_length,
            integer_memory_length: self.integer_memory_length,
            string_memory_length: self.string_memory_length,
            list_memory_length: self.list_memory_length,
            function_memory_length: self.function_memory_length,
            prototype_index: self.prototype_index,
        }
    }
}

impl<'a> Chunk<'a> for FullChunk {
    fn chunk_type_name() -> &'static str {
        "Full Chunk"
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
        Some(&self.positions)
    }

    fn constants(&self) -> &[Value<Self>] {
        &self.constants
    }

    fn arguments(&self) -> &[Vec<(Address, OperandType)>] {
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

impl<'a> From<ChunkCompiler<'a, Self>> for FullChunk {
    fn from(compiler: ChunkCompiler<'a, Self>) -> Self {
        compiler.finish()
    }
}

impl Disassemble for FullChunk {
    fn disassembler<'a, 'w, W: std::io::Write>(
        &'a self,
        writer: &'w mut W,
    ) -> Disassembler<'a, 'w, Self, W> {
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
            && self.constants == other.constants
            && self.locals == other.locals
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
            && self.constants == other.constants
            && self.locals == other.locals
            && self.arguments == other.arguments
            && self.prototype_index == other.prototype_index
            && self.boolean_memory_length == other.boolean_memory_length
            && self.byte_memory_length == other.byte_memory_length
            && self.character_memory_length == other.character_memory_length
            && self.float_memory_length == other.float_memory_length
            && self.integer_memory_length == other.integer_memory_length
            && self.string_memory_length == other.string_memory_length
            && self.list_memory_length == other.list_memory_length
            && self.function_memory_length == other.function_memory_length
    }
}

impl PartialOrd for FullChunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FullChunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .as_ref()
            .cmp(&other.name.as_ref())
            .then_with(|| self.r#type.cmp(&other.r#type))
            .then_with(|| self.instructions.cmp(&other.instructions))
            .then_with(|| self.positions.cmp(&other.positions))
            .then_with(|| self.constants.cmp(&other.constants))
            .then_with(|| self.locals.iter().cmp(other.locals.iter()))
            .then_with(|| self.arguments.cmp(&other.arguments))
            .then_with(|| self.prototype_index.cmp(&other.prototype_index))
    }
}
