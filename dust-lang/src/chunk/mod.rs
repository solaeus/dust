//! Representation of a Dust program or function.
//!
//! **Except for testing purposes, a chunk should not be created directly. Instead, use the compiler
//! to generate a chunk from Dust source code.**
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

use std::fmt::{self, Debug, Display, Formatter, Write as FmtWrite};
use std::io::Write;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{DustString, Function, FunctionType, Instruction, Span, Type, TypeCode};

/// A tuple that represents the value and type arguments passed to a function when called. Each tuple
/// in this list corresponds to a function call. The first item in each tuple represents the
/// registers of the values passed to the call. The second item is the call's type arguments.
pub type ArgumentLists = (Vec<(u16, TypeCode)>, Vec<Type>);

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Default, PartialOrd, Serialize, Deserialize)]
pub struct Chunk {
    pub name: Option<DustString>,
    pub r#type: FunctionType,

    pub instructions: Vec<Instruction>,
    pub positions: Vec<Span>,
    pub character_constants: Vec<char>,
    pub float_constants: Vec<f64>,
    pub integer_constants: Vec<i64>,
    pub string_constants: Vec<DustString>,
    pub locals: Vec<Local>,
    pub prototypes: Vec<Arc<Chunk>>,
    pub argument_lists: Vec<ArgumentLists>,

    pub boolean_memory_length: u16,
    pub byte_memory_length: u16,
    pub character_memory_length: u16,
    pub float_memory_length: u16,
    pub integer_memory_length: u16,
    pub string_memory_length: u16,
    pub list_memory_length: u16,
    pub function_memory_length: u16,
    pub prototype_index: u16,
}

impl Chunk {
    pub fn as_function(&self) -> Function {
        Function {
            prototype_index: self.prototype_index,
        }
    }

    pub fn disassembler<'a, W: Write>(&'a self, writer: &'a mut W) -> Disassembler<'a, W> {
        Disassembler::new(self, writer)
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
            f.write_char('\n')?; // Improves readability in Cargo test output
        }

        write!(f, "{string}")
    }
}

impl Eq for Chunk {}

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
    }
}
