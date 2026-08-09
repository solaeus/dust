use std::fmt::{self, Display, Formatter};

use dust_compiler::{
    constants::ConstantsError,
    dust_type::DustType,
    instruction::{MemoryKind, OperandType, Operation},
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
    InvalidInstructionDispatch {
        index: usize,
    },
    InvalidInstructionPointer {
        instruction_pointer: u32,
    },
    InvalidCharacter {
        value: u32,
    },
    InvalidOperandDispatch,
    InvalidOperationDispatch,
}

impl VmError {
    fn message(&self) -> String {
        match self {
            Self::ConstantList(constant_list_error) => format!("{constant_list_error:?}"),
            Self::ChannelError => "Channel error".to_string(),
            Self::InvalidPrototypeIndex { index } => format!("Invalid prototype index: {index}"),
            Self::UnsupportedOperation { operation } => {
                format!("Unsupported operation: {operation:?}")
            }
            Self::UnsupportedMemoryKind { memory } => {
                format!("Unsupported memory kind: {memory:?}")
            }
            Self::UnsupportedOperandType { operand_type } => {
                format!("Unsupported operand type: {operand_type:?}")
            }
            Self::CallStackUnderflow => "Call stack underflow".to_string(),
            Self::InvalidReturnValue {
                register_count,
                expected_type,
            } => format!(
                "Invalid return value: register_count={register_count}, expected_type={expected_type:?}"
            ),
            Self::InvalidRegisterIndex { index } => format!("Invalid register index: {index}"),
            Self::InvalidInstructionDispatch { index } => {
                format!("Invalid instruction dispatch: {index}")
            }
            Self::InvalidInstructionPointer {
                instruction_pointer: index,
            } => {
                format!("Invalid instruction pointer: {index}")
            }
            Self::InvalidCharacter { value } => {
                format!("Invalid character bits: {value}")
            }
            Self::InvalidOperandDispatch => "Invalid operand dispatch".to_string(),
            Self::InvalidOperationDispatch => "Invalid operation dispatch".to_string(),
        }
    }
}

impl From<ConstantsError> for VmError {
    fn from(constant_list_error: ConstantsError) -> Self {
        Self::ConstantList(constant_list_error)
    }
}

impl Display for VmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "VM Error: {}", self.message())
    }
}
