//! The Dust programming language.
//!
//! Dust is a statically typed, interpreted programming language.
pub mod abstract_tree;
pub mod analyzer;
pub mod built_in_function;
pub mod context;
pub mod dust_error;
pub mod identifier;
pub mod lex;
pub mod parse;
pub mod token;
pub mod r#type;
pub mod value;
pub mod vm;

pub use abstract_tree::{AbstractSyntaxTree, BinaryOperator, Node, Statement};
pub use analyzer::{analyze, Analyzer, AnalyzerError};
pub use built_in_function::{BuiltInFunction, BuiltInFunctionError};
pub use context::{Context, UsageData, VariableData};
pub use dust_error::DustError;
pub use identifier::Identifier;
pub use lex::{lex, LexError, Lexer};
pub use parse::{parse, ParseError, Parser};
pub use r#type::Type;
pub use token::Token;
pub use value::{Value, ValueError};
pub use vm::{run, Vm, VmError};

pub type Span = (usize, usize);
