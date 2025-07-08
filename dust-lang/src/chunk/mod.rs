//! Representation of a Dust program or function.
//!
//! A chunk is output by the compiler to represent all the information needed to execute a Dust
//! program. In addition to the program itself, each function in the source is compiled into its own
//! chunk and stored in the `prototypes` field of its parent. Thus, a chunk can also represent a
//! function prototype.
//!
//! Chunks have a name when they belong to a named function. They also have a type, so the input
//! parameters and the type of the return value are statically known.
mod debug_chunk;
mod disassembler;
mod stripped_chunk;

pub use debug_chunk::DebugChunk;
pub use disassembler::Disassembler;
pub use stripped_chunk::StrippedChunk;

use std::sync::Arc;
use std::{fmt::Debug, io::Write};

use crate::{FunctionType, Instruction, Local, Path, Value, compiler::CompiledData};

pub trait Chunk:
    Sized + Clone + Debug + Default + Eq + PartialEq + PartialOrd + Ord + Disassemble
{
    fn new(data: CompiledData<Self>) -> Self;

    fn chunk_type_name() -> &'static str;

    fn name(&self) -> Option<&Path>;

    fn r#type(&self) -> &FunctionType;

    fn instructions(&self) -> &Vec<Instruction>;

    fn constants(&self) -> &[Value<Self>];

    fn locals(&self) -> Option<impl Iterator<Item = (&Path, &Local)>>;

    fn register_count(&self) -> usize;

    fn prototype_index(&self) -> usize;

    fn into_function(self) -> Arc<Self>;
}

pub trait Disassemble: Sized {
    fn disassembler<'a, 'w, W: Write>(&'a self, writer: &'w mut W)
    -> Disassembler<'a, 'w, Self, W>;
}
