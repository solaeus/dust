//! Representation of a Dust program or function.
//!
//! A prototype is output by the compiler to represent all the information needed to execute a function.
//! Each function in the source is compiled into its own prototype and stored in the global `prototypes`
//! collection.

use std::fmt::Debug;

use crate::{
    instruction::{Address, Instruction, OperandType},
    source::Position,
    r#type::FunctionType,
};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Prototype {
    pub(crate) index: u16,
    pub(crate) name_position: Option<Position>,
    pub(crate) function_type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) call_arguments: Vec<(Address, OperandType)>,
    pub(crate) drop_lists: Vec<u16>,

    pub(crate) register_count: u16,
    pub(crate) is_recursive: bool,
}
