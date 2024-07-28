/**
The Dust programming language.

Dust is a statically typed, interpreted programming language.

The [interpreter] module contains the `Interpreter` struct, which is used to lex, parse and/or
interpret Dust code. The `interpret` function is a convenience function that creates a new
`Interpreter` and runs the given source code.
*/
pub mod abstract_tree;
pub mod context;
pub mod error;
pub mod identifier;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod standard_library;
pub mod value;

pub use abstract_tree::Type;
pub use value::Value;
