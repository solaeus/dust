//! Representation of a Dust function.
//!
//! A prototype is output by the compiler to represent all the information needed to execute a function.
use std::fmt::Debug;

use crate::{
    compiler::Symbol,
    instruction::{Address, Instruction, OperandType},
    r#type::FunctionType,
};

/// Representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prototype {
    pub(crate) name: Symbol,
    pub(crate) index: u16,
    pub(crate) function_type: FunctionType,

    pub(crate) instructions: Vec<Instruction>,
    pub(crate) call_arguments: Vec<(Address, OperandType)>,
    pub(crate) drops: Vec<u16>,

    pub(crate) register_count: u16,
}

impl Prototype {
    pub fn dummy() -> Self {
        Self {
            name: Symbol::BuiltIn("dummy_prototype"),
            index: 0,
            function_type: FunctionType::default(),
            instructions: vec![],
            call_arguments: vec![],
            drops: vec![],
            register_count: 0,
        }
    }
}

impl Default for Prototype {
    fn default() -> Self {
        Self::dummy()
    }
}
