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
mod local;
mod scope;

pub use disassembler::Disassembler;
pub use local::Local;
pub use scope::Scope;

use std::fmt::{self, Debug, Display, Formatter};
use std::io::Write;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::instruction::AddressKind;
use crate::value::AbstractFunction;
use crate::{Address, DustString, FunctionType, Instruction, Span, Type};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Default, PartialOrd, Serialize, Deserialize)]
pub struct Chunk {
    pub(crate) name: Option<DustString>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) positions: Vec<Span>,
    pub(crate) character_constants: Vec<char>,
    pub(crate) float_constants: Vec<f64>,
    pub(crate) integer_constants: Vec<i64>,
    pub(crate) string_constants: Vec<DustString>,
    pub(crate) locals: Vec<Local>,
    pub(crate) prototypes: Vec<Arc<Chunk>>,
    pub(crate) arguments: Vec<Arguments>,

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

impl Chunk {
    pub fn as_function(&self) -> AbstractFunction {
        AbstractFunction {
            prototype_address: Address::new(self.prototype_index, AddressKind::FUNCTION_PROTOTYPE),
        }
    }

    pub fn disassembler<'a, W: Write>(&'a self, writer: &'a mut W) -> Disassembler<'a, W> {
        Disassembler::new(self, writer)
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut output = Vec::new();

        self.disassembler(&mut output)
            .style(true)
            .disassemble()
            .unwrap();

        let string = String::from_utf8_lossy(&output);

        write!(f, "{string}")
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = Vec::new();

        self.disassembler(&mut output)
            .style(true)
            .disassemble()
            .unwrap();

        let string = String::from_utf8_lossy(&output);

        if cfg!(debug_assertions) {
            writeln!(f)?; // Improves readability in Cargo test output
        }

        write!(f, "{string}")
    }
}

impl Eq for Chunk {}

/// For testing purposes, ignore the "memory_length" fields so that we don't have to write them them
/// when writing Chunks for tests.
#[cfg(debug_assertions)]
impl PartialEq for Chunk {
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
impl PartialEq for Chunk {
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

/// Represents the value and type arguments passed to a function when called.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Arguments {
    pub values: Vec<Address>,
    pub types: Vec<Type>,
}
