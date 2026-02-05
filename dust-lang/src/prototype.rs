//! Representation of a Dust function.
//!
//! A prototype is output by the compiler to represent all the information needed to execute a function.
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
    pub(crate) drops: Vec<u16>,

    pub(crate) register_count: u16,
}
