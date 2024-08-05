/**
The Dust programming language.

Dust is a statically typed, interpreted programming language.

The [interpreter] module contains the `Interpreter` struct, which is used to lex, parse and/or
interpret Dust code. The `interpret` function is a convenience function that creates a new
`Interpreter` and runs the given source code.
*/
pub mod identifier;
pub mod instruction;
pub mod lex;
pub mod parse;
pub mod token;
pub mod r#type;
pub mod value;
pub mod vm;

pub use identifier::Identifier;
pub use instruction::{Instruction, Operation};
pub use lex::{lex, LexError, Lexer};
pub use parse::{parse, ParseError, Parser};
pub use r#type::Type;
pub use token::Token;
pub use value::{Value, ValueError};
pub use vm::Vm;

pub type Span = (usize, usize);
