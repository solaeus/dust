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
    Address, DustString, FunctionType, Instruction, Local, Span, compiler::ChunkCompiler,
    r#type::TypeKind,
};

pub trait Chunk: Sized + Clone + Debug + Disassemble {
    fn from_chunk_compiler<'a>(compiler: ChunkCompiler<'a, Self>) -> Self;

    fn chunk_type_name() -> &'static str;

    fn name(&self) -> Option<&DustString>;

    fn r#type(&self) -> &FunctionType;

    fn as_function(self) -> Arc<Self>;

    fn instructions(&self) -> &[Instruction];

    fn positions(&self) -> Option<&[Span]>;

    fn character_constants(&self) -> &[char];

    fn float_constants(&self) -> &[f64];

    fn integer_constants(&self) -> &[i64];

    fn string_constants(&self) -> &[DustString];

    fn prototypes(&self) -> &[Arc<Self>];

    fn arguments(&self) -> &[Vec<(Address, TypeKind)>];

    fn locals(&self) -> Option<impl Iterator<Item = (&DustString, &Local)>>;

    fn boolean_memory_length(&self) -> u16;

    fn byte_memory_length(&self) -> u16;

    fn character_memory_length(&self) -> u16;

    fn float_memory_length(&self) -> u16;

    fn integer_memory_length(&self) -> u16;

    fn string_memory_length(&self) -> u16;

    fn list_memory_length(&self) -> u16;

    fn function_memory_length(&self) -> u16;

    fn prototype_index(&self) -> u16;
}

pub trait Disassemble: Sized {
    fn disassembler<'a, W: Write>(&'a self, writer: &'a mut W) -> Disassembler<'a, Self, W>;
}
