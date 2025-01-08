use std::fmt::{self, Display, Formatter};

use crate::{InstructionData, Value};

use super::{stack::Stack, FunctionCall};

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    CallStackUnderflow,
    ExpectedFunction {
        value: Value,
    },
    InstructionIndexOutOfBounds {
        call_stack: Stack<FunctionCall>,
        ip: usize,
    },
    MalformedInstruction {
        instruction: InstructionData,
    },
}

impl Display for VmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::CallStackUnderflow => {
                write!(f, "Call stack underflow")
            }
            Self::ExpectedFunction { value } => {
                write!(f, "Expected function, found {value}")
            }
            Self::InstructionIndexOutOfBounds { call_stack, ip } => {
                write!(f, "Instruction index {} out of bounds\n{call_stack}", ip)
            }
            Self::MalformedInstruction { instruction } => {
                write!(f, "Malformed instruction {instruction}")
            }
        }
    }
}
