use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    Disassembler, FunctionType, Instruction, List, Local, Path, Value, chunk::Disassemble,
    compiler::CompiledData,
};

use super::{Chunk, StrippedChunk};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FullChunk {
    pub(crate) name: Option<Path>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) constants: Vec<Value<Self>>,
    pub(crate) locals: IndexMap<Path, Local>,

    pub(crate) register_count: usize,
    pub(crate) prototype_index: usize,
}

impl FullChunk {
    pub fn strip(self) -> StrippedChunk {
        StrippedChunk {
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
                                .map(|list| list.strip_chunks())
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
            register_count: self.register_count,
            prototype_index: self.prototype_index,
        }
    }
}

impl Chunk for FullChunk {
    fn new(data: CompiledData<Self>) -> Self {
        FullChunk {
            name: data.name,
            r#type: data.r#type,
            instructions: data.instructions,
            constants: data.constants,
            locals: data.locals,
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

    fn constants(&self) -> &[Value<Self>] {
        &self.constants
    }

    fn locals(&self) -> Option<impl Iterator<Item = (&Path, &Local)>> {
        Some(self.locals.iter())
    }

    fn register_count(&self) -> usize {
        self.register_count
    }

    fn prototype_index(&self) -> usize {
        self.prototype_index
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

impl Eq for FullChunk {}

/// For testing purposes, ignore the "memory_length" fields so that we don't have to write them them
/// when writing Chunks for tests.
#[cfg(debug_assertions)]
impl PartialEq for FullChunk {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.r#type == other.r#type
            && self.instructions == other.instructions
            && self.constants == other.constants
            && self.locals == other.locals
            && self.prototype_index == other.prototype_index
    }
}

#[cfg(not(debug_assertions))]
impl PartialEq for FullChunk {
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
            .then_with(|| self.constants.cmp(&other.constants))
            .then_with(|| self.locals.iter().cmp(other.locals.iter()))
            .then_with(|| self.register_count.cmp(&other.register_count))
            .then_with(|| self.prototype_index.cmp(&other.prototype_index))
    }
}
