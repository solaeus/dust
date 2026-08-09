use std::fmt::{self, Display, Formatter};

use dust_compiler::{
    constants::ConstantsError,
    dust_type::DustType,
    instruction::{Instruction, MemoryKind, OperandType, Operation},
};

#[derive(Debug)]
pub enum VmError {
    CallStackUnderflow,
    ConstantList(ConstantsError),
    ChannelError,
    InvalidPrototypeIndex {
        index: u32,
    },
    UnsupportedOperation {
        operation: Operation,
    },
    UnsupportedMemoryKind {
        memory: MemoryKind,
    },
    UnsupportedOperandType {
        operand_type: OperandType,
    },
    InvalidReturnValue {
        register_count: usize,
        expected_type: DustType,
    },
    InvalidRegisterIndex {
        index: usize,
    },
    InvalidInstructionPointer {
        instruction_pointer: usize,
    },
    InvalidCharacter {
        value: u32,
    },
    InvalidInstructionDispatch {
        instruction: Instruction,
    },
}

impl From<ConstantsError> for VmError {
    fn from(constant_list_error: ConstantsError) -> Self {
        Self::ConstantList(constant_list_error)
    }
}

impl Display for VmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ConstantList(constant_list_error) => write!(f, "{constant_list_error:?}"),
            Self::ChannelError => write!(f, "Channel error"),
            Self::InvalidPrototypeIndex { index } => write!(f, "Invalid prototype index: {index}"),
            Self::UnsupportedOperation { operation } => {
                write!(f, "Unsupported operation: {operation:?}")
            }
            Self::UnsupportedMemoryKind { memory } => {
                write!(f, "Unsupported memory kind: {memory:?}")
            }
            Self::UnsupportedOperandType { operand_type } => {
                write!(f, "Unsupported operand type: {operand_type:?}")
            }
            Self::CallStackUnderflow => write!(f, "Call stack underflow"),
            Self::InvalidReturnValue {
                register_count,
                expected_type,
            } => write!(
                f,
                "Invalid return value: register_count={register_count}, expected_type={expected_type:?}"
            ),
            Self::InvalidRegisterIndex { index } => write!(f, "Invalid register index: {index}"),
            Self::InvalidInstructionPointer {
                instruction_pointer: index,
            } => {
                write!(f, "Invalid instruction pointer: {index}")
            }
            Self::InvalidCharacter { value } => {
                write!(f, "Invalid character bits: {value}")
            }
            Self::InvalidInstructionDispatch { instruction } => {
                write!(f, "Invalid instruction dispatch: {instruction}")
            }
        }
    }
}
