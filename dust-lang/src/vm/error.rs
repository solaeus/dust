use std::fmt::{self, Display, Formatter};

use crate::DustString;

use super::call_stack::CallStack;

#[derive(Clone, Debug, PartialEq)]
pub enum VmError {
    CallStackUnderflow { thread_name: DustString },
    InstructionIndexOutOfBounds { call_stack: CallStack, ip: usize },
    MalformedInstruction { instruction: InstructionData },
}

impl Display for VmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::CallStackUnderflow { thread_name } => {
                write!(f, "Call stack underflow in thread {thread_name}")
            }
            Self::InstructionIndexOutOfBounds { call_stack, ip } => {
                write!(f, "Instruction index {} out of bounds\n{call_stack}", ip)
            }
        }
    }
}
