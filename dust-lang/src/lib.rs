//! The Dust programming language.
//!
//! To get started, you can use the `run` function to run a Dust program.
//!
//! ```rust
//! use dust_lang::{run, Value};
//!
//! let program = "
//!     foo = 21
//!     bar = 2
//!     foo * bar
//! ";
//!
//! let the_answer = run(program).unwrap();
//!
//! assert_eq!(the_answer, Some(Value::integer(42)));
//! ```
pub mod abstract_tree;
pub mod analyzer;
pub mod built_in_function;
pub mod context;
pub mod dust_error;
pub mod identifier;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod r#type;
pub mod value;
pub mod vm;

pub use abstract_tree::{
    AbstractSyntaxTree, AssignmentOperator, BinaryOperator, Node, Statement, StructDefinition,
    StructInstantiation, UnaryOperator,
};
pub use analyzer::{analyze, Analyzer, AnalyzerError};
pub use built_in_function::{BuiltInFunction, BuiltInFunctionError};
pub use context::{Context, VariableData};
pub use dust_error::DustError;
pub use identifier::Identifier;
pub use lexer::{lex, LexError, Lexer};
pub use parser::{parse, ParseError, Parser};
pub use r#type::{StructType, Type};
pub use token::{Token, TokenKind, TokenOwned};
pub use value::{Struct, Value, ValueError};
pub use vm::{run, run_with_context, Vm, VmError};

pub type Span = (usize, usize);
