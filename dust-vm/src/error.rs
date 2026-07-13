use crate::{
    constants::ConstantsError,
    dust_type::DustType,
    error::DustError,
    instruction::{MemoryKind, OperandType, Operation},
};

#[derive(Debug)]
pub enum VmError {
    ConstantList(ConstantsError),

    InvalidPrototypeIndex {
        index: u16,
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
    CallStackUnderflow,
    InvalidReturnValue {
        register_count: usize,
        expected_type: DustType,
    },
    InvalidRegisterIndex {
        index: u16,
    },
}

impl From<ConstantsError> for VmError {
    fn from(constant_list_error: ConstantsError) -> Self {
        Self::ConstantList(constant_list_error)
    }
}

impl<'a> DustError<'a> for VmError {
    type Info = ();

    fn add_report(&self, _: Self::Info, groups: &mut Vec<annotate_snippets::Group<'a>>) {
        match self {
            VmError::ConstantList(constant_list_error) => {
                constant_list_error.add_report((), groups);
            }
            VmError::InvalidPrototypeIndex { .. }
            | VmError::UnsupportedMemoryKind { .. }
            | VmError::UnsupportedOperation { .. }
            | VmError::UnsupportedOperandType { .. }
            | VmError::CallStackUnderflow
            | VmError::InvalidReturnValue { .. }
            | VmError::InvalidRegisterIndex { .. } => self.add_internal_report(groups),
        }
    }
}
