use crate::{
    constants::ConstantListError,
    dust_type::DustType,
    error::AnnotatedError,
    instruction::{MemoryKind, OperandType, Operation},
};

#[derive(Debug)]
pub enum VmError {
    ConstantList(ConstantListError),

    InvalidPrototypeId {
        prototype_id: u16,
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

impl From<ConstantListError> for VmError {
    fn from(constant_list_error: ConstantListError) -> Self {
        Self::ConstantList(constant_list_error)
    }
}

impl<'a> AnnotatedError<'a> for VmError {
    type Context = ();

    fn add_report(&self, _: Self::Context, groups: &mut Vec<annotate_snippets::Group<'a>>) {
        match self {
            VmError::ConstantList(constant_list_error) => {
                constant_list_error.add_report((), groups);
            }
            VmError::InvalidPrototypeId { .. }
            | VmError::UnsupportedMemoryKind { .. }
            | VmError::UnsupportedOperation { .. }
            | VmError::UnsupportedOperandType { .. }
            | VmError::CallStackUnderflow
            | VmError::InvalidReturnValue { .. }
            | VmError::InvalidRegisterIndex { .. } => self.add_internal_report(groups),
        }
    }
}
