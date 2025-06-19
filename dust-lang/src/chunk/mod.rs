//! Representation of a Dust program or function.
//!
//! A chunk is output by the compiler to represent all the information needed to execute a Dust
//! program. In addition to the program itself, each function in the source is compiled into its own
//! chunk and stored in the `prototypes` field of its parent. Thus, a chunk can also represent a
//! function prototype.
//!
//! Chunks have a name when they belong to a named function. They also have a type, so the input
//! parameters and the type of the return value are statically known.
mod disassembler;
mod full_chunk;
mod stripped_chunk;

pub use disassembler::Disassembler;
pub use full_chunk::FullChunk;
pub use stripped_chunk::StrippedChunk;

use std::sync::Arc;
use std::{fmt::Debug, io::Write};

use crate::{
    Address, FunctionType, Instruction, Local, OperandType, Path, Span, Value,
    compiler::CompiledData,
};

pub trait Chunk:
    Sized + Clone + Debug + Default + Eq + PartialEq + PartialOrd + Ord + Disassemble
{
    fn new(data: CompiledData<Self>) -> Self;

    fn chunk_type_name() -> &'static str;

    fn name(&self) -> Option<&Path>;

    fn r#type(&self) -> &FunctionType;

    fn instructions(&self) -> &[Instruction];

    fn positions(&self) -> Option<&[Span]>;

    fn constants(&self) -> &[Value<Self>];

    fn arguments(&self) -> &[Vec<(Address, OperandType)>];

    fn locals(&self) -> Option<impl Iterator<Item = (&Path, &Local)>>;

    fn boolean_memory_length(&self) -> u16;

    fn byte_memory_length(&self) -> u16;

    fn character_memory_length(&self) -> u16;

    fn float_memory_length(&self) -> u16;

    fn integer_memory_length(&self) -> u16;

    fn string_memory_length(&self) -> u16;

    fn list_memory_length(&self) -> u16;

    fn function_memory_length(&self) -> u16;

    fn prototype_index(&self) -> u16;

    fn into_function(self) -> Arc<Self>;
}

pub trait Disassemble: Sized {
    fn disassembler<'a, 'w, W: Write>(&'a self, writer: &'w mut W)
    -> Disassembler<'a, 'w, Self, W>;
}
