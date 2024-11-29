//! The Dust programming language library.

pub mod chunk;
pub mod compiler;
pub mod disassembler;
pub mod dust_error;
pub mod instruction;
pub mod lexer;
pub mod native_function;
pub mod operation;
pub mod optimize;
pub mod scope;
pub mod token;
pub mod r#type;
pub mod value;
pub mod vm;

pub use crate::chunk::{Chunk, Local};
pub use crate::compiler::{compile, CompileError, Compiler};
pub use crate::disassembler::Disassembler;
pub use crate::dust_error::{AnnotatedError, DustError};
pub use crate::instruction::{Argument, Destination, Instruction};
pub use crate::lexer::{lex, LexError, Lexer};
pub use crate::native_function::{NativeFunction, NativeFunctionError};
pub use crate::operation::Operation;
pub use crate::optimize::{optimize_control_flow, optimize_set_local};
pub use crate::r#type::{EnumType, FunctionType, StructType, Type, TypeConflict};
pub use crate::scope::Scope;
pub use crate::token::{display_token_list, Token, TokenKind, TokenOwned};
pub use crate::value::{AbstractValue, ConcreteValue, RangeValue, Value, ValueError, ValueRef};
pub use crate::vm::{run, Vm, VmError};

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span(pub usize, pub usize);

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
