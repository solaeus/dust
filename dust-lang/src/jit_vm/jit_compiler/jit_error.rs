use crate::{ErrorMessage, MemoryKind, OperandType, Operation, Span, dust_error::AnnotatedError};

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
    ConstantIndexOutOfBounds {
        constant_index: usize,
        total_constant_count: usize,
    },
    SafepointIndexOutOfBounds {
        safepoint_index: usize,
        total_safepoint_count: usize,
    },
}

impl AnnotatedError for JitError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "JIT Compilation Error";

        let (description, detail_snippets, help_snippet) = match self {
            JitError::ArgumentsIndexOutOfBounds {
                arguments_index,
                total_argument_count,
            } => (
                "Arguments index out of bounds",
                vec![(
                    format!(
                        "Arguments index {arguments_index} is out of bounds for total argument count {total_argument_count}."
                    ),
                    Span::default(),
                )],
                None,
            ),
            JitError::JumpToSelf { ip } => (
                "Jump to self detected",
                vec![(
                    format!("Jump to self detected at instruction pointer {ip}."),
                    Span::default(),
                )],
                None,
            ),
            JitError::JumpTargetOutOfBounds {
                target_instruction_pointer,
                total_instruction_count,
            } => (
                "Jump target out of bounds",
                vec![(
                    format!(
                        "Jump target {target_instruction_pointer} is out of bounds for total instruction count {total_instruction_count}."
                    ),
                    Span::default(),
                )],
                None,
            ),
            JitError::BranchTargetOutOfBounds {
                branch_target_instruction_pointer,
                total_instruction_count,
            } => (
                "Branch target out of bounds",
                vec![(
                    format!(
                        "Branch target {branch_target_instruction_pointer} is out of bounds for total instruction count {total_instruction_count}."
                    ),
                    Span::default(),
                )],
                None,
            ),
            JitError::InvalidConstantType { expected_type } => (
                "Invalid constant type",
                vec![(
                    format!("Constant expected type was {expected_type}."),
                    Span::default(),
                )],
                None,
            ),
            JitError::UnsupportedOperandType { operand_type } => (
                "Unsupported operand type",
                vec![(
                    format!("Unsupported operand type: {operand_type}."),
                    Span::default(),
                )],
                None,
            ),
            JitError::UnsupportedMemoryKind { memory_kind } => (
                "Unsupported memory kind",
                vec![(
                    format!("Unsupported memory kind: {memory_kind}."),
                    Span::default(),
                )],
                None,
            ),
            JitError::UnhandledOperation { operation } => (
                "Unhandled operation",
                vec![(
                    format!("Unhandled operation: {operation}."),
                    Span::default(),
                )],
                None,
            ),
            JitError::CraneliftModuleError { message } => (
                "Cranelift module error",
                vec![(
                    format!("Cranelift module error: {message}."),
                    Span::default(),
                )],
                None,
            ),
            JitError::FunctionCompilationError {
                message,
                cranelift_ir,
            } => (
                "Function compilation error",
                vec![(
                    format!("Function compilation error: {message}\nCranelift IR:\n{cranelift_ir}"),
                    Span::default(),
                )],
                None,
            ),
            JitError::FunctionIndexOutOfBounds {
                ip,
                function_index,
                total_function_count,
            } => (
                "Function index out of bounds",
                vec![(
                    format!(
                        "Function index {function_index} at instruction pointer {ip} is out of bounds for total function count {total_function_count}."
                    ),
                    Span::default(),
                )],
                None,
            ),
            JitError::RegisterIndexOutOfBounds {
                register_index,
                total_register_count,
            } => (
                "Register index out of bounds",
                vec![(
                    format!(
                        "Register index {register_index} is out of bounds for total register count {total_register_count}."
                    ),
                    Span::default(),
                )],
                None,
            ),
            JitError::ConstantIndexOutOfBounds {
                constant_index,
                total_constant_count,
            } => (
                "Constant index out of bounds",
                vec![(
                    format!(
                        "Constant index {constant_index} is out of bounds for total constant count {total_constant_count}."
                    ),
                    Span::default(),
                )],
                None,
            ),
            JitError::SafepointIndexOutOfBounds {
                safepoint_index,
                total_safepoint_count,
            } => (
                "Safepoint index out of bounds",
                vec![(
                    format!(
                        "Safepoint index {safepoint_index} is out of bounds for total safepoint count {total_safepoint_count}."
                    ),
                    Span::default(),
                )],
                None,
            ),
        };

        ErrorMessage {
            title,
            description,
            detail_snippets,
            help_snippet,
        }
    }
}
