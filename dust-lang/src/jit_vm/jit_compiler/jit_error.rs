use crate::{MemoryKind, OperandType, Operation, Span, dust_error::AnnotatedError};

pub const JIT_ERROR_TEXT: &str = "An error occurred during JIT compilation.";

#[derive(Clone, Debug, PartialEq)]
pub enum JitError {
    ArgumentsIndexOutOfBounds {
        arguments_index: usize,
        total_argument_count: usize,
    },
    JumpToSelf {
        ip: usize,
    },
    JumpTargetOutOfBounds {
        target_instruction_pointer: isize,
        total_instruction_count: usize,
    },
    BranchTargetOutOfBounds {
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
        operation: Operation,
    },
    CraneliftModuleError {
        message: String,
    },
    FunctionCompilationError {
        message: String,
        cranelift_ir: String,
    },
    FunctionIndexOutOfBounds {
        ip: usize,
        function_index: usize,
        total_function_count: usize,
    },
    RegisterIndexOutOfBounds {
        register_index: usize,
        total_register_count: usize,
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
            JitError::FunctionIndexOutOfBounds { .. } => "Function index out of bounds",
            JitError::RegisterIndexOutOfBounds { .. } => "Register index out of bounds",
        }
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        vec![match self {
            JitError::ArgumentsIndexOutOfBounds {
                arguments_index,
                total_argument_count,
            } => (
                format!(
                    "Arguments index {arguments_index} is out of bounds for total argument count {total_argument_count}."
                ),
                Span::default(),
            ),
            JitError::JumpToSelf { ip } => (
                format!("Jump to self detected at instruction pointer {ip}."),
                Span::default(),
            ),
            JitError::JumpTargetOutOfBounds {
                target_instruction_pointer,
                total_instruction_count,
            } => (
                format!(
                    "Jump target {target_instruction_pointer} is out of bounds for total instruction count {total_instruction_count}."
                ),
                Span::default(),
            ),
            JitError::BranchTargetOutOfBounds {
                branch_target_instruction_pointer,
                total_instruction_count,
            } => (
                format!(
                    "Branch target {branch_target_instruction_pointer} is out of bounds for total instruction count {total_instruction_count}."
                ),
                Span::default(),
            ),
            JitError::InvalidConstantType {
                constant_index,
                expected_type,
            } => (
                format!("Constant index {constant_index} expected type was {expected_type}."),
                Span::default(),
            ),
            JitError::UnsupportedOperandType { operand_type } => (
                format!("Unsupported operand type: {operand_type}."),
                Span::default(),
            ),
            JitError::UnsupportedMemoryKind { memory_kind } => (
                format!("Unsupported memory kind: {memory_kind}."),
                Span::default(),
            ),
            JitError::UnhandledOperation { operation } => (
                format!("Unhandled operation: {operation}."),
                Span::default(),
            ),
            JitError::CraneliftModuleError { message } => (
                format!("Cranelift module error: {message}."),
                Span::default(),
            ),
            JitError::FunctionCompilationError {
                message,
                cranelift_ir,
            } => (
                format!("Function compilation error: {message}\nCranelift IR:\n{cranelift_ir}"),
                Span::default(),
            ),
            JitError::FunctionIndexOutOfBounds {
                ip,
                function_index,
                total_function_count,
            } => (
                format!(
                    "Function index {function_index} at instruction pointer {ip} is out of bounds for total function count {total_function_count}."
                ),
                Span::default(),
            ),
            JitError::RegisterIndexOutOfBounds {
                register_index,
                total_register_count,
            } => (
                format!(
                    "Register index {register_index} is out of bounds for total register count {total_register_count}."
                ),
                Span::default(),
            ),
        }]
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        Vec::new()
    }
}
