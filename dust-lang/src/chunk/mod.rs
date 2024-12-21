//! Representation of a Dust program or function.
//!
//! A chunk is output by the compiler to represent all of the information needed to execute a Dust
//! program. In addition to the program itself, each function in the source is compiled into its own
//! chunk and stored in the `prototypes` field of its parent. Thus, a chunk is also the
//! representation of a function prototype, i.e. a function declaration, as opposed to an individual
//! instance.
//!
//! Chunks have a name when they belong to a named function. They also have a type, so the input
//! parameters and the type of the return value are statically known. The [`Chunk::stack_size`]
//! method can provide the necessary stack size that will be needed by the virtual machine. Chunks
//! cannot be instantiated directly and must be created by the compiler. However, when the Rust
//! compiler is in the "test" configuration (used for all types of test), [`Chunk::with_data`] can
//! be used to create a chunk for comparison to the compiler output. Do not try to run these chunks
//! in a virtual machine. Due to their missing stack size and record index, they will cause a panic.
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

use crate::vm::{Record, RunAction};
use crate::{DustString, Function, FunctionType, Instruction, Span, Value};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, PartialOrd, Serialize, Deserialize)]
pub struct Chunk {
    name: Option<DustString>,
    r#type: FunctionType,

    instructions: SmallVec<[Instruction; 32]>,
    positions: SmallVec<[Span; 32]>,
    constants: SmallVec<[Value; 16]>,
    locals: SmallVec<[Local; 8]>,
    prototypes: Vec<Chunk>,

    stack_size: usize,
    record_index: u8,
}

impl Chunk {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: Option<DustString>,
        r#type: FunctionType,
        instructions: impl Into<SmallVec<[Instruction; 32]>>,
        positions: impl Into<SmallVec<[Span; 32]>>,
        constants: impl Into<SmallVec<[Value; 16]>>,
        locals: impl Into<SmallVec<[Local; 8]>>,
        prototypes: impl Into<Vec<Chunk>>,
        stack_size: usize,
        record_index: u8,
    ) -> Self {
        Self {
            name,
            r#type,
            instructions: instructions.into(),
            positions: positions.into(),
            constants: constants.into(),
            locals: locals.into(),
            prototypes: prototypes.into(),
            stack_size,
            record_index,
        }
    }

    #[cfg(any(test, debug_assertions))]
    pub fn with_data(
        name: Option<DustString>,
        r#type: FunctionType,
        instructions: impl Into<SmallVec<[Instruction; 32]>>,
        positions: impl Into<SmallVec<[Span; 32]>>,
        constants: impl Into<SmallVec<[Value; 16]>>,
        locals: impl Into<SmallVec<[Local; 8]>>,
        prototypes: Vec<Chunk>,
    ) -> Self {
        Self {
            name,
            r#type,
            instructions: instructions.into(),
            positions: positions.into(),
            constants: constants.into(),
            locals: locals.into(),
            prototypes,
            stack_size: 0,
            record_index: 0,
        }
    }

    pub fn as_function(&self) -> Function {
        Function {
            name: self.name.clone(),
            r#type: self.r#type.clone(),
            record_index: self.record_index,
        }
    }

    pub fn into_records(self, records: &mut Vec<Record>) {
        let actions = self.instructions().iter().map(RunAction::from).collect();
        let record = Record::new(
            actions,
            None,
            self.name,
            self.r#type,
            self.positions,
            self.constants,
            self.locals,
            self.stack_size,
            self.record_index,
        );

        if records.is_empty() {
            records.push(record);

            for chunk in self.prototypes {
                chunk.into_records(records);
            }
        } else {
            for chunk in self.prototypes {
                chunk.into_records(records);
            }

            debug_assert!(record.index() as usize == records.len());

            records.push(record);
        }
    }

    pub fn name(&self) -> Option<&DustString> {
        self.name.as_ref()
    }

    pub fn r#type(&self) -> &FunctionType {
        &self.r#type
    }

    pub fn instructions(&self) -> &SmallVec<[Instruction; 32]> {
        &self.instructions
    }

    pub fn positions(&self) -> &SmallVec<[Span; 32]> {
        &self.positions
    }

    pub fn constants(&self) -> &SmallVec<[Value; 16]> {
        &self.constants
    }

    pub fn locals(&self) -> &SmallVec<[Local; 8]> {
        &self.locals
    }

    pub fn prototypes(&self) -> &Vec<Chunk> {
        &self.prototypes
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
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
