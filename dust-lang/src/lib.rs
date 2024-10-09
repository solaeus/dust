mod chunk;
mod constructor;
mod dust_error;
mod identifier;
mod instruction;
mod lexer;
mod operation;
mod parser;
mod token;
mod r#type;
mod value;
mod vm;

use std::fmt::Display;

pub use chunk::{Chunk, ChunkDisassembler, ChunkError, Local};
pub use constructor::Constructor;
pub use dust_error::{AnnotatedError, DustError};
pub use identifier::Identifier;
pub use instruction::Instruction;
pub use lexer::{lex, LexError, Lexer};
pub use operation::Operation;
pub use parser::{parse, ParseError, Parser};
pub use r#type::{EnumType, FunctionType, RangeableType, StructType, Type, TypeConflict};
pub use token::{Token, TokenKind, TokenOwned};
pub use value::{Enum, Function, Struct, Value, ValueError};
pub use vm::{run, Vm, VmError};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span(pub usize, pub usize);

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
