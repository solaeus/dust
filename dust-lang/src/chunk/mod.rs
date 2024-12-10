//! In-memory representation of a Dust program or function.
//!
//! A chunk consists of a sequence of instructions and their positions, a list of constants, and a
//! list of locals that can be executed by the Dust virtual machine. Chunks have a name when they
//! belong to a named function.
mod disassembler;
mod local;
mod scope;

pub use disassembler::Disassembler;
pub use local::Local;
pub use scope::Scope;

use std::fmt::{self, Debug, Display, Formatter, Write as FmtWrite};
use std::io::Write;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{ConcreteValue, DustString, FunctionType, Instruction, Span, Type};

/// In-memory representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, PartialOrd, Serialize, Deserialize)]
pub struct Chunk {
    name: Option<DustString>,
    r#type: FunctionType,

    instructions: SmallVec<[(Instruction, Span); 32]>,
    constants: SmallVec<[ConcreteValue; 16]>,
    locals: SmallVec<[Local; 8]>,
}

impl Chunk {
    pub fn new(name: Option<DustString>) -> Self {
        Self {
            name,
            instructions: SmallVec::new(),
            constants: SmallVec::new(),
            locals: SmallVec::new(),
            r#type: FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::None,
            },
        }
    }
    pub fn with_data(
        name: Option<DustString>,
        r#type: FunctionType,
        instructions: impl Into<SmallVec<[(Instruction, Span); 32]>>,
        constants: impl Into<SmallVec<[ConcreteValue; 16]>>,
        locals: impl Into<SmallVec<[Local; 8]>>,
    ) -> Self {
        Self {
            name,
            r#type,
            instructions: instructions.into(),
            constants: constants.into(),
            locals: locals.into(),
        }
    }

    pub fn name(&self) -> Option<&DustString> {
        self.name.as_ref()
    }

    pub fn r#type(&self) -> &FunctionType {
        &self.r#type
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn constants(&self) -> &SmallVec<[ConcreteValue; 16]> {
        &self.constants
    }

    pub fn instructions(&self) -> &SmallVec<[(Instruction, Span); 32]> {
        &self.instructions
    }

    pub fn locals(&self) -> &SmallVec<[Local; 8]> {
        &self.locals
    }

    pub fn stack_size(&self) -> usize {
        self.instructions()
            .iter()
            .rev()
            .find_map(|(instruction, _)| {
                if instruction.yields_value() {
                    Some(instruction.a as usize + 1)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub fn disassembler<'a, W: Write>(&'a self, writer: &'a mut W) -> Disassembler<W> {
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
        self.instructions == other.instructions
            && self.constants == other.constants
            && self.locals == other.locals
    }
}
