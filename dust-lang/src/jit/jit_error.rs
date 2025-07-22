use std::fmt::{self, Display, Formatter};

use crate::{OperandType, Span, dust_error::AnnotatedError};

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
        instruction_pointer: usize,
        constant_index: usize,
        expected_type: OperandType,
        operation: String,
    },
    UnsupportedOperandType {
        instruction_pointer: usize,
        operand_type: OperandType,
        operation: String,
    },
    UnsupportedMemoryKind {
        instruction_pointer: usize,
        operation: String,
        memory_kind_description: String,
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

impl Display for JitError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            JitError::JumpToSelf {
                instruction_pointer,
            } => {
                write!(
                    formatter,
                    "JIT compilation error: Jump to self detected at instruction pointer {instruction_pointer}"
                )
            }
            JitError::JumpTargetOutOfBounds {
                instruction_pointer,
                target_instruction_pointer,
                total_instruction_count,
            } => {
                write!(
                    formatter,
                    "JIT compilation error: Jump target out of bounds at instruction pointer {instruction_pointer}. Target {target_instruction_pointer} is outside valid range [0, {total_instruction_count})"
                )
            }
            JitError::BranchTargetOutOfBounds {
                instruction_pointer,
                branch_target_instruction_pointer,
                total_instruction_count,
            } => {
                write!(
                    formatter,
                    "JIT compilation error: Branch target out of bounds at instruction pointer {instruction_pointer}. Target {branch_target_instruction_pointer} is outside valid range [0, {total_instruction_count})"
                )
            }
            JitError::InvalidConstantType {
                instruction_pointer,
                constant_index,
                expected_type,
                operation,
            } => {
                write!(
                    formatter,
                    "JIT compilation error: Invalid constant type at instruction pointer {instruction_pointer}. Constant at index {constant_index} cannot be used as {expected_type:?} in {operation} operation"
                )
            }
            JitError::UnsupportedOperandType {
                instruction_pointer,
                operand_type,
                operation,
            } => {
                write!(
                    formatter,
                    "JIT compilation error: Unsupported operand type {operand_type:?} at instruction pointer {instruction_pointer} in {operation} operation"
                )
            }
            JitError::UnsupportedMemoryKind {
                instruction_pointer,
                operation,
                memory_kind_description,
            } => {
                write!(
                    formatter,
                    "JIT compilation error: Unsupported memory kind '{memory_kind_description}' at instruction pointer {instruction_pointer} in {operation} operation"
                )
            }
            JitError::UnhandledOperation {
                instruction_pointer,
                operation_name,
            } => {
                write!(
                    formatter,
                    "JIT compilation error: Unhandled operation '{operation_name}' at instruction pointer {instruction_pointer}"
                )
            }
            JitError::CraneliftModuleError {
                instruction_pointer,
                message,
            } => match instruction_pointer {
                Some(ip) => write!(
                    formatter,
                    "JIT compilation error: Cranelift module error at instruction pointer {ip}: {message}"
                ),
                None => write!(
                    formatter,
                    "JIT compilation error: Cranelift module error: {message}"
                ),
            },
            JitError::FunctionCompilationError { message } => {
                write!(
                    formatter,
                    "JIT compilation error: Function compilation failed: {message}"
                )
            }
        }
    }
}

impl std::error::Error for JitError {}

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
