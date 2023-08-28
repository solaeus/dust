//! The Dust library is used to implement the Dust language, `src/main.rs` implements the command
//! line binary.
//!
//! Using this library is simple and straightforward, see the [inferface] module for instructions on
//! interpreting Dust code. Most of the language's features are implemented in the [tools] module.
#![forbid(unsafe_code)]

pub use crate::{
    error::*,
    interface::*,
    operator::Operator,
    token::PartialToken,
    tools::{Tool, ToolInfo, TOOL_LIST},
    tree::Node,
    value::{
        function::Function, table::Table, time::Time, value_type::ValueType,
        variable_map::VariableMap, Value,
    },
};

pub mod tools;

mod error;
mod interface;
mod operator;
mod token;
mod tree;
mod value;
