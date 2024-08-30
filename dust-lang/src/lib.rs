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
pub mod analyzer;
pub mod ast;
pub mod built_in_function;
pub mod constructor;
pub mod context;
pub mod core_library;
pub mod dust_error;
pub mod evaluation;
pub mod identifier;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod r#type;
pub mod value;
pub mod vm;

pub use analyzer::{analyze, AnalysisError, Analyzer};
pub use ast::{AbstractSyntaxTree, AstError, Expression, Node, Span, Statement};
pub use built_in_function::{BuiltInFunction, BuiltInFunctionError};
pub use constructor::Constructor;
pub use context::{Context, ContextData, ContextError};
pub use core_library::core_library;
pub use dust_error::DustError;
pub use evaluation::{Evaluation, TypeEvaluation};
pub use identifier::Identifier;
pub use lexer::{lex, LexError, Lexer};
pub use parser::{parse, ParseError, Parser};
pub use r#type::*;
pub use token::{Token, TokenKind, TokenOwned};
pub use value::*;
pub use vm::{run, RuntimeError, Vm};
