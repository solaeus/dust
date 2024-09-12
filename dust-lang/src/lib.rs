mod chunk;
mod constructor;
mod dust_error;
mod identifier;
mod instruction;
mod lexer;
mod parser;
mod token;
mod r#type;
mod value;
mod vm;

use std::fmt::Display;

pub use chunk::{Chunk, ChunkError};
pub use constructor::Constructor;
pub use dust_error::{AnnotatedError, DustError};
pub use identifier::Identifier;
pub use instruction::{Instruction, Operation};
pub use lexer::{lex, LexError, Lexer};
pub use parser::{parse, ParseError, Parser};
pub use r#type::{EnumType, FunctionType, RangeableType, StructType, Type, TypeConflict};
pub use token::{Token, TokenKind, TokenOwned};
pub use value::{Enum, Function, Struct, Value, ValueError};
pub use vm::{run, Vm, VmError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span(pub usize, pub usize);

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
