//! The Dust programming language.
//!
//! To get started, you can use the `run` function to run a Dust program.
//!
//! ```rust
//! use dust_lang::{run, Value};
//!
//! let program = "
//!     let foo = 21
//!     let bar = 2
//!     foo * bar
//! ";
//!
//! let the_answer = run(program).unwrap();
//!
//! assert_eq!(the_answer, Some(Value::integer(42)));
//! ```
pub mod chunk;
pub mod constructor;
pub mod dust_error;
pub mod identifier;
pub mod identifier_stack;
pub mod lexer;
pub mod parser;
pub mod run;
pub mod token;
pub mod r#type;
pub mod value;
pub mod vm;

pub use chunk::{Chunk, ChunkError};
pub use constructor::{ConstructError, Constructor};
pub use dust_error::DustError;
pub use identifier::Identifier;
pub use identifier_stack::IdentifierStack;
pub use lexer::{lex, LexError, Lexer};
pub use parser::{parse, ParseError, Parser};
pub use r#type::{EnumType, FunctionType, RangeableType, StructType, Type, TypeConflict};
pub use run::run;
pub use token::{Token, TokenKind, TokenOwned};
pub use value::{Struct, Value, ValueError};
pub use vm::{Instruction, Vm};

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Span(pub usize, pub usize);

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
