use crate::{MemoryKind, OperandType, Span, dust_error::AnnotatedError};

#[derive(Clone, Debug, PartialEq)]
pub enum JitError {
    JumpToSelf {
        instruction_pointer: usize,
    },
    JumpTargetOutOfBounds {
        instruction_pointer: usize,
        target_instruction_pointer: usize,
        total_instruction_count: usize,
    },
    BranchTargetOutOfBounds {
        instruction_pointer: usize,
        branch_target_instruction_pointer: usize,
        total_instruction_count: usize,
    },
    InvalidConstantType {
        constant_index: usize,
        expected_type: OperandType,
    },
    UnsupportedOperandType {
        operand_type: OperandType,
    },
    UnsupportedMemoryKind {
        memory_kind: MemoryKind,
    },
    UnhandledOperation {
        instruction_pointer: usize,
        operation_name: String,
    },
    CraneliftModuleError {
        instruction_pointer: Option<usize>,
        message: String,
    },
    FunctionCompilationError {
        message: String,
    },
}

impl AnnotatedError for JitError {
    fn title(&self) -> &'static str {
        "JIT Compilation Error"
    }

    fn description(&self) -> &'static str {
        match self {
            JitError::JumpToSelf { .. } => "Jump to self detected",
            JitError::JumpTargetOutOfBounds { .. } => "Jump target out of bounds",
            JitError::BranchTargetOutOfBounds { .. } => "Branch target out of bounds",
            JitError::InvalidConstantType { .. } => "Invalid constant type",
            JitError::UnsupportedOperandType { .. } => "Unsupported operand type",
            JitError::UnsupportedMemoryKind { .. } => "Unsupported memory kind",
            JitError::UnhandledOperation { .. } => "Unhandled operation",
            JitError::CraneliftModuleError { .. } => "Cranelift module error",
            JitError::FunctionCompilationError { .. } => "Function compilation error",
        }
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        Vec::new()
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        Vec::new()
    }
}
