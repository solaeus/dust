//! Representation of a Dust program or function.
//!
//! A chunk is output by the compiler to represent all the information needed to execute a Dust
//! program. In addition to the program itself, each function in the source is compiled into its own
//! chunk and stored in the `prototypes` field of its parent. Thus, a chunk can also represent a
//! function prototype.
//!
//! Chunks have a name when they belong to a named function. They also have a type, so the input
//! parameters and the type of the return value are statically known. The [`Chunk::stack_size`]
//! field can provide the necessary stack size that will be needed by the virtual machine. Chunks
//! cannot be instantiated directly and must be created by the compiler. However, when the Rust
//! compiler is in the "test" or "debug_assertions" configuration (used for all types of test),
//! [`Chunk::with_data`] can be used to create a chunk for comparison to the compiler output. Do not
//! try to run these chunks in a virtual machine. Due to their missing stack size and record index,
//! they will cause a panic or undefined behavior.
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

use crate::{ConcreteValue, DustString, Function, FunctionType, Instruction, Span};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, PartialOrd, Serialize, Deserialize)]
pub struct Chunk {
    pub(crate) name: Option<DustString>,
    pub(crate) r#type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) positions: Vec<Span>,
    pub(crate) constants: Vec<ConcreteValue>,
    pub(crate) locals: Vec<Local>,
    pub(crate) prototypes: Vec<Arc<Chunk>>,

    pub(crate) boolean_register_count: usize,
    pub(crate) byte_register_count: usize,
    pub(crate) character_register_count: usize,
    pub(crate) float_register_count: usize,
    pub(crate) integer_register_count: usize,
    pub(crate) string_register_count: usize,
    pub(crate) list_register_count: usize,
    pub(crate) function_register_count: usize,
    pub(crate) prototype_index: u16,
}

impl Chunk {
    pub fn as_function(&self) -> Function {
        Function {
            name: self.name.clone(),
            r#type: self.r#type.clone(),
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
            && self.constants == other.constants
            && self.locals == other.locals
            && self.prototypes == other.prototypes
    }
}
