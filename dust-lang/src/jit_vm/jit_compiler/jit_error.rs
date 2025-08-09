use crate::{Instruction, MemoryKind, OperandType, Operation, Span, dust_error::AnnotatedError};

pub const JIT_ERROR_TEXT: &str = "An error occurred during JIT compilation.";

#[derive(Clone, Debug, PartialEq)]
pub enum JitError {
    ArgumentsIndexOutOfBounds {
        ip: usize,
        arguments_index: usize,
        total_argument_count: usize,
    },
    JumpToSelf {
        ip: usize,
    },
    JumpTargetOutOfBounds {
        ip: usize,
        target_instruction_pointer: isize,
        total_instruction_count: usize,
    },
    BranchTargetOutOfBounds {
        ip: usize,
        branch_target_instruction_pointer: usize,
        total_instruction_count: usize,
    },
    InvalidConstantType {
        ip: usize,
        instruction: Instruction,
        constant_index: usize,
        expected_type: OperandType,
    },
    UnsupportedOperandType {
        ip: usize,
        instruction: Instruction,
        operand_type: OperandType,
    },
    UnsupportedMemoryKind {
        ip: usize,
        instruction: Instruction,
        memory_kind: MemoryKind,
    },
    UnhandledOperation {
        ip: usize,
        instruction: Instruction,
        operation: Operation,
    },
    CraneliftModuleError {
        message: String,
    },
    FunctionCompilationError {
        message: String,
        cranelift_ir: String,
    },
}

impl AnnotatedError for JitError {
    fn title(&self) -> &'static str {
        "JIT Compilation Error"
    }

    fn description(&self) -> &'static str {
        match self {
            JitError::ArgumentsIndexOutOfBounds { .. } => "Arguments index out of bounds",
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
            JitError::ArgumentsIndexOutOfBounds {
                ip,
                arguments_index,
                total_argument_count,
            } => (
                format!(
                    "Arguments index out of bounds at ip {ip}: index {arguments_index}, total arguments {total_argument_count}"
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::JumpToSelf { ip } => (
                format!("Jump to self detected at ip {ip}"),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::JumpTargetOutOfBounds {
                ip,
                target_instruction_pointer,
                total_instruction_count,
            } => (
                format!(
                    "Jump target out of bounds at ip {ip}, target {target_instruction_pointer}, total instructions {total_instruction_count}"
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::BranchTargetOutOfBounds {
                ip,
                branch_target_instruction_pointer,
                total_instruction_count,
            } => (
                format!(
                    "Branch target out of bounds at ip {ip}, branch target {branch_target_instruction_pointer}, total instructions {total_instruction_count}"
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::InvalidConstantType {
                ip,
                instruction,
                constant_index,
                expected_type,
            } => (
                format!(
                    "Invalid constant type at ip {ip} ({instruction}): index {constant_index}, expected {expected_type}"
                ),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::UnsupportedOperandType {
                ip,
                instruction,
                operand_type,
            } => (
                format!("Unsupported operand type at ip {ip} ({instruction}): {operand_type}"),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::UnsupportedMemoryKind {
                ip,
                instruction,
                memory_kind,
            } => (
                format!("Unsupported memory kind at ip {ip} ({instruction}): {memory_kind}"),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::UnhandledOperation {
                ip,
                instruction,
                operation,
            } => (
                format!("Unhandled operation at ip {ip} ({instruction}): {operation}"),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
            JitError::CraneliftModuleError { message } => {
                (message.clone(), Span(0, JIT_ERROR_TEXT.len()))
            }
            JitError::FunctionCompilationError {
                message,
                cranelift_ir,
            } => (
                format!("{message}\nCranelift IR:\n{cranelift_ir}"),
                Span(0, JIT_ERROR_TEXT.len()),
            ),
        }]
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        Vec::new()
    }
}
