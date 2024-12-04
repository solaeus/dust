//! In-memory representation of a Dust program or function.
//!
//! A chunk consists of a sequence of instructions and their positions, a list of constants, and a
//! list of locals that can be executed by the Dust virtual machine. Chunks have a name when they
//! belong to a named function.

use std::fmt::{self, Debug, Display, Write};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use smartstring::alias::String;

use crate::{ConcreteValue, Disassembler, FunctionType, Instruction, Scope, Span, Type};

/// In-memory representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, PartialOrd, Serialize, Deserialize)]
pub struct Chunk {
    name: Option<String>,
    r#type: FunctionType,

    instructions: SmallVec<[Instruction; 32]>,
    positions: SmallVec<[Span; 32]>,
    constants: SmallVec<[ConcreteValue; 16]>,
    locals: SmallVec<[Local; 8]>,
}

impl Chunk {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            instructions: SmallVec::new(),
            positions: SmallVec::new(),
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
        name: Option<String>,
        r#type: FunctionType,
        instructions: SmallVec<[Instruction; 32]>,
        positions: SmallVec<[Span; 32]>,
        constants: SmallVec<[ConcreteValue; 16]>,
        locals: SmallVec<[Local; 8]>,
    ) -> Self {
        Self {
            name,
            r#type,
            instructions,
            positions,
            constants,
            locals,
        }
    }

    pub fn name(&self) -> Option<&String> {
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

    pub fn instructions(&self) -> &SmallVec<[Instruction; 32]> {
        &self.instructions
    }

    pub fn positions(&self) -> &SmallVec<[Span; 32]> {
        &self.positions
    }

    pub fn locals(&self) -> &SmallVec<[Local; 8]> {
        &self.locals
    }

    pub fn stack_size(&self) -> usize {
        self.instructions()
            .iter()
            .rev()
            .find_map(|instruction| {
                if instruction.yields_value() {
                    Some(instruction.a() as usize + 1)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub fn disassembler(&self) -> Disassembler {
        Disassembler::new(self)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disassembly = self.disassembler().style(true).disassemble();

        write!(f, "{disassembly}")
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disassembly = self.disassembler().style(false).disassemble();

        if cfg!(debug_assertions) {
            f.write_char('\n')?;
        }

        write!(f, "{}", disassembly)
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

/// A scoped variable.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    /// The index of the identifier in the constants table.
    pub identifier_index: u16,

    /// The expected type of the local's value.
    pub r#type: Type,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Scope where the variable was declared.
    pub scope: Scope,
}

impl Local {
    /// Creates a new Local instance.
    pub fn new(identifier_index: u16, r#type: Type, mutable: bool, scope: Scope) -> Self {
        Self {
            identifier_index,
            r#type,
            is_mutable: mutable,
            scope,
        }
    }
}
