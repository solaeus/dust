use crate::{MemoryKind, OperandType, Operation, Span, dust_error::AnnotatedError};

pub const JIT_ERROR_TEXT: &str = "An error occurred during JIT compilation.";

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
        operation: Operation,
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
        vec![match self {
            JitError::JumpToSelf {
                instruction_pointer,
            } => (
                format!(
                    "Jump to self detected at instruction pointer: {}",
                    instruction_pointer
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::JumpTargetOutOfBounds {
                instruction_pointer,
                target_instruction_pointer,
                total_instruction_count,
            } => (
                format!(
                    "Jump target out of bounds at instruction pointer: {}, target: {}, total instructions: {}",
                    instruction_pointer, target_instruction_pointer, total_instruction_count
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::BranchTargetOutOfBounds {
                instruction_pointer,
                branch_target_instruction_pointer,
                total_instruction_count,
            } => (
                format!(
                    "Branch target out of bounds at instruction pointer: {}, branch target: {}, total instructions: {}",
                    instruction_pointer, branch_target_instruction_pointer, total_instruction_count
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::InvalidConstantType {
                constant_index,
                expected_type,
            } => (
                format!(
                    "Invalid constant type at index {}. Expected type: {:?}",
                    constant_index, expected_type
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::UnsupportedOperandType { operand_type } => (
                format!("Unsupported operand type: {:?}", operand_type),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::UnsupportedMemoryKind { memory_kind } => (
                format!("Unsupported memory kind: {:?}", memory_kind),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::UnhandledOperation {
                instruction_pointer,
                operation,
            } => (
                format!(
                    "Unhandled operation at instruction pointer {}: {}",
                    instruction_pointer, operation
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::CraneliftModuleError {
                instruction_pointer,
                message,
            } => (
                format!(
                    "Cranelift module error{}: {}",
                    instruction_pointer.map_or("".to_string(), |ip| format!(
                        " at instruction pointer {}",
                        ip
                    )),
                    message
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::FunctionCompilationError { message } => (
                format!("Function compilation error: {}", message),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
        }]
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        Vec::new()
    }
}
