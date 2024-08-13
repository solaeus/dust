//! The Dust programming language.
//!
//! Dust is a statically typed, interpreted programming language.
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
    AbstractSyntaxTree, BinaryOperator, Node, Statement, StructDefinition, UnaryOperator,
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
