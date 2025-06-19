use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    Address, FunctionType, Instruction, Local, OperandType, Path, Span, Value,
    compiler::CompiledData,
};

use super::{Chunk, Disassemble, Disassembler};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct StrippedChunk {
    pub(crate) name: Option<Path>,
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

impl Chunk for StrippedChunk {
    fn new(data: CompiledData<Self>) -> Self {
        StrippedChunk {
            name: data.name,
            r#type: data.r#type,
            instructions: data.instructions,
            constants: data.constants,
            arguments: data.arguments,
            boolean_memory_length: data.boolean_memory_length,
            byte_memory_length: data.byte_memory_length,
            character_memory_length: data.character_memory_length,
            float_memory_length: data.float_memory_length,
            integer_memory_length: data.integer_memory_length,
            string_memory_length: data.string_memory_length,
            list_memory_length: data.list_memory_length,
            function_memory_length: data.function_memory_length,
            prototype_index: data.prototype_index,
        }
    }

    fn chunk_type_name() -> &'static str {
        "Stripped Chunk"
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

    fn locals(&self) -> Option<impl Iterator<Item = (&Path, &Local)>> {
        None::<std::iter::Empty<(&Path, &Local)>>
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

impl<'de> Deserialize<'de> for StrippedChunk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrippedChunkVisitor<'de> {
            _marker: std::marker::PhantomData<&'de ()>,
        }

        impl<'de> serde::de::Visitor<'de> for StrippedChunkVisitor<'de> {
            type Value = StrippedChunk;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a StrippedChunk struct")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut name = None;
                let mut r#type = FunctionType::default();
                let mut instructions = Vec::new();
                let mut constants = Vec::new();
                let mut arguments = Vec::new();
                let mut boolean_memory_length = 0;
                let mut byte_memory_length = 0;
                let mut character_memory_length = 0;
                let mut float_memory_length = 0;
                let mut integer_memory_length = 0;
                let mut string_memory_length = 0;
                let mut list_memory_length = 0;
                let mut function_memory_length = 0;
                let mut prototype_index = 0;

                while let Some(key) = access.next_key::<&str>()? {
                    match key {
                        "name" => {
                            name = access.next_value()?;
                        }
                        "r#type" => {
                            r#type = access.next_value()?;
                        }
                        "instructions" => {
                            instructions = access.next_value()?;
                        }
                        "constants" => {
                            constants = access.next_value()?;
                        }
                        "arguments" => {
                            arguments = access.next_value()?;
                        }
                        "boolean_memory_length" => {
                            boolean_memory_length = access.next_value()?;
                        }
                        "byte_memory_length" => {
                            byte_memory_length = access.next_value()?;
                        }
                        "character_memory_length" => {
                            character_memory_length = access.next_value()?;
                        }
                        "float_memory_length" => {
                            float_memory_length = access.next_value()?;
                        }
                        "integer_memory_length" => {
                            integer_memory_length = access.next_value()?;
                        }
                        "string_memory_length" => {
                            string_memory_length = access.next_value()?;
                        }
                        "list_memory_length" => {
                            list_memory_length = access.next_value()?;
                        }
                        "function_memory_length" => {
                            function_memory_length = access.next_value()?;
                        }
                        "prototype_index" => {
                            prototype_index = access.next_value()?;
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                key,
                                &[
                                    "name",
                                    "r#type",
                                    "instructions",
                                    "constants",
                                    "arguments",
                                    "boolean_memory_length",
                                    "byte_memory_length",
                                    "character_memory_length",
                                    "float_memory_length",
                                    "integer_memory_length",
                                    "string_memory_length",
                                    "list_memory_length",
                                    "function_memory_length",
                                    "prototype_index",
                                ],
                            ));
                        }
                    }
                }

                Ok(StrippedChunk {
                    name,
                    r#type,
                    instructions,
                    constants,
                    arguments,
                    boolean_memory_length,
                    byte_memory_length,
                    character_memory_length,
                    float_memory_length,
                    integer_memory_length,
                    string_memory_length,
                    list_memory_length,
                    function_memory_length,
                    prototype_index,
                })
            }
        }

        deserializer.deserialize_struct(
            "StrippedChunk",
            &[
                "name",
                "r#type",
                "instructions",
                "constants",
                "arguments",
                "boolean_memory_length",
                "byte_memory_length",
                "character_memory_length",
                "float_memory_length",
                "integer_memory_length",
                "string_memory_length",
                "list_memory_length",
                "function_memory_length",
                "prototype_index",
            ],
            StrippedChunkVisitor {
                _marker: PhantomData,
            },
        )
    }
}
