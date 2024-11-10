//! In-memory representation of a Dust program or function.
//!
//! A chunk consists of a sequence of instructions and their positions, a list of constants, and a
//! list of locals that can be executed by the Dust virtual machine. Chunks have a name when they
//! belong to a named function.

use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Disassembler, Instruction, Span, Type, Value};

/// In-memory representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Chunk {
    name: Option<String>,

    instructions: Vec<(Instruction, Span)>,
    constants: Vec<Value>,
    locals: Vec<Local>,

    return_type: Type,
    current_scope: Scope,
    block_index: u8,
}

impl Chunk {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: Vec::new(),
            return_type: Type::None,
            current_scope: Scope::default(),
            block_index: 0,
        }
    }

    pub fn with_data(
        name: Option<String>,
        instructions: Vec<(Instruction, Span)>,
        constants: Vec<Value>,
        locals: Vec<Local>,
    ) -> Self {
        Self {
            name,
            instructions,
            constants,
            locals,
            return_type: Type::None,
            current_scope: Scope::default(),
            block_index: 0,
        }
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn constants(&self) -> &Vec<Value> {
        &self.constants
    }

    pub fn constants_mut(&mut self) -> &mut Vec<Value> {
        &mut self.constants
    }

    pub fn take_constants(self) -> Vec<Value> {
        self.constants
    }

    pub fn instructions(&self) -> &Vec<(Instruction, Span)> {
        &self.instructions
    }

    pub fn instructions_mut(&mut self) -> &mut Vec<(Instruction, Span)> {
        &mut self.instructions
    }

    pub fn get_instruction(&self, index: usize) -> Result<&(Instruction, Span), ChunkError> {
        self.instructions
            .get(index)
            .ok_or(ChunkError::InstructionIndexOutOfBounds { index })
    }

    pub fn locals(&self) -> &Vec<Local> {
        &self.locals
    }

    pub fn locals_mut(&mut self) -> &mut Vec<Local> {
        &mut self.locals
    }

    pub fn get_local(&self, index: u8) -> Result<&Local, ChunkError> {
        self.locals
            .get(index as usize)
            .ok_or(ChunkError::LocalIndexOutOfBounds {
                index: index as usize,
            })
    }

    pub fn get_local_mut(&mut self, index: u8) -> Result<&mut Local, ChunkError> {
        self.locals
            .get_mut(index as usize)
            .ok_or(ChunkError::LocalIndexOutOfBounds {
                index: index as usize,
            })
    }

    pub fn current_scope(&self) -> Scope {
        self.current_scope
    }

    pub fn get_constant(&self, index: u8) -> Result<&Value, ChunkError> {
        self.constants
            .get(index as usize)
            .ok_or(ChunkError::ConstantIndexOutOfBounds {
                index: index as usize,
            })
    }

    pub fn push_or_get_constant(&mut self, value: Value) -> u8 {
        if let Some(index) = self
            .constants
            .iter()
            .position(|constant| constant == &value)
        {
            return index as u8;
        }

        self.constants.push(value);

        (self.constants.len() - 1) as u8
    }

    pub fn get_identifier(&self, local_index: u8) -> Option<String> {
        self.locals.get(local_index as usize).and_then(|local| {
            self.constants
                .get(local.identifier_index as usize)
                .map(|value| value.to_string())
        })
    }

    pub fn begin_scope(&mut self) {
        self.block_index += 1;
        self.current_scope.block_index = self.block_index;
        self.current_scope.depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.current_scope.depth -= 1;

        if self.current_scope.depth == 0 {
            self.current_scope.block_index = 0;
        } else {
            self.current_scope.block_index -= 1;
        }
    }

    pub fn get_constant_type(&self, constant_index: u8) -> Result<Type, ChunkError> {
        self.constants
            .get(constant_index as usize)
            .map(|value| value.r#type())
            .ok_or(ChunkError::ConstantIndexOutOfBounds {
                index: constant_index as usize,
            })
    }

    pub fn get_local_type(&self, local_index: u8) -> Result<&Type, ChunkError> {
        self.locals
            .get(local_index as usize)
            .map(|local| &local.r#type)
            .ok_or(ChunkError::LocalIndexOutOfBounds {
                index: local_index as usize,
            })
    }

    pub fn return_type(&self) -> &Type {
        &self.return_type
    }

    pub fn disassembler(&self) -> Disassembler {
        Disassembler::new(self)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disassembler = self.disassembler().styled(false);

        write!(f, "{}", disassembler.disassemble())
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disassembly = self.disassembler().styled(false).disassemble();

        if cfg!(debug_assertions) {
            write!(f, "\n{}", disassembly)
        } else {
            write!(f, "{}", disassembly)
        }
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
    pub identifier_index: u8,

    /// The expected type of the local's value.
    pub r#type: Type,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Scope where the variable was declared.
    pub scope: Scope,
}

impl Local {
    /// Creates a new Local instance.
    pub fn new(identifier_index: u8, r#type: Type, mutable: bool, scope: Scope) -> Self {
        Self {
            identifier_index,
            r#type,
            is_mutable: mutable,
            scope,
        }
    }
}

/// Variable locality, as defined by its depth and block index.
///
/// The `block index` is a unique identifier for a block within a chunk. It is used to differentiate
/// between blocks that are not nested together but have the same depth, i.e. sibling scopes. If the
/// `block_index` is 0, then the scope is the root scope of the chunk. The `block_index` is always 0
/// when the `depth` is 0. See [Chunk::begin_scope][] and [Chunk::end_scope][] to see how scopes are
/// incremented and decremented.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Scope {
    /// Level of block nesting.
    pub depth: u8,
    /// Index of the block in the chunk.
    pub block_index: u8,
}

impl Scope {
    pub fn new(depth: u8, block_index: u8) -> Self {
        Self { depth, block_index }
    }

    pub fn contains(&self, other: &Self) -> bool {
        match self.depth.cmp(&other.depth) {
            Ordering::Less => false,
            Ordering::Greater => self.block_index >= other.block_index,
            Ordering::Equal => self.block_index == other.block_index,
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.depth, self.block_index)
    }
}

/// Errors that can occur when using a [`Chunk`].
#[derive(Clone, Debug, PartialEq)]
pub enum ChunkError {
    ConstantIndexOutOfBounds { index: usize },
    InstructionIndexOutOfBounds { index: usize },
    LocalIndexOutOfBounds { index: usize },
}

impl Display for ChunkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ChunkError::ConstantIndexOutOfBounds { index } => {
                write!(f, "Constant index {} out of bounds", index)
            }
            ChunkError::InstructionIndexOutOfBounds { index } => {
                write!(f, "Instruction index {} out of bounds", index)
            }
            ChunkError::LocalIndexOutOfBounds { index } => {
                write!(f, "Local index {} out of bounds", index)
            }
        }
    }
}
