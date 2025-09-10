use annotate_snippets::{Group, Level};
use cranelift_module::ModuleError;

use crate::{MemoryKind, OperandType, Operation, dust_error::AnnotatedError};

pub const JIT_ERROR_TEXT: &str = "An error occurred during JIT compilation.";

#[derive(Debug)]
pub enum JitError {
    CompilationError {
        message: String,
        cranelift_ir: String,
    },
    UnsupportedOperandType {
        operand_type: OperandType,
    },
    DropListRangeOutOfBounds {
        drop_list_start: u16,
        drop_list_end: u16,
        total_safepoint_count: usize,
    },
    UnhandledOperation {
        operation: Operation,
    },
    UnsupportedMemoryKind {
        memory_kind: MemoryKind,
    },
    FunctionIndexOutOfBounds {
        ip: usize,
        function_index: u16,
        total_function_count: usize,
    },
    ArgumentsRangeOutOfBounds {
        arguments_list_start: u16,
        arguments_list_end: u16,
        total_argument_count: usize,
    },
    ConstantIndexOutOfBounds {
        constant_index: u16,
        total_constant_count: usize,
    },
    InvalidConstantType {
        expected_type: OperandType,
    },
    RegisterIndexOutOfBounds {
        register_index: u16,
        total_register_count: usize,
    },
    CraneliftModuleError {
        error: Box<ModuleError>,
        cranelift_ir: String,
    },
}

impl AnnotatedError for JitError {
    fn annotated_error<'a>(&'a self, _source: &'a str) -> Group<'a> {
        match self {
            JitError::CompilationError { message, .. } => {
                let title = format!("JIT compilation failed: {message}");
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::UnsupportedOperandType { operand_type } => {
                let title = format!("Unsupported operand type for JIT: {:?}", operand_type);
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::DropListRangeOutOfBounds {
                drop_list_start,
                drop_list_end,
                total_safepoint_count,
            } => {
                let title = format!(
                    "Drop list range [{}, {}) is out of bounds (safepoints: {})",
                    drop_list_start, drop_list_end, total_safepoint_count
                );
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::UnhandledOperation { operation } => {
                let title = format!("Unhandled operation in JIT: {:?}", operation);
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::UnsupportedMemoryKind { memory_kind } => {
                let title = format!("Unsupported memory kind in JIT: {:?}", memory_kind);
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::FunctionIndexOutOfBounds {
                ip,
                function_index,
                total_function_count,
            } => {
                let title = format!(
                    "Function index {} out of bounds at instruction pointer {} (total functions: {})",
                    function_index, ip, total_function_count
                );
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::ArgumentsRangeOutOfBounds {
                arguments_list_start,
                arguments_list_end,
                total_argument_count,
            } => {
                let title = format!(
                    "Arguments list range [{}, {}) is out of bounds (total arguments: {})",
                    arguments_list_start, arguments_list_end, total_argument_count
                );
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::ConstantIndexOutOfBounds {
                constant_index,
                total_constant_count,
            } => {
                let title = format!(
                    "Constant index {} out of bounds (total constants: {})",
                    constant_index, total_constant_count
                );
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::InvalidConstantType { expected_type } => {
                let title = format!("Invalid constant type; expected {:?}", expected_type);
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::RegisterIndexOutOfBounds {
                register_index,
                total_register_count,
            } => {
                let title = format!(
                    "Register index {} out of bounds (total registers: {})",
                    register_index, total_register_count
                );
                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::CraneliftModuleError {
                error,
                cranelift_ir,
            } => {
                let title = format!("Cranelift module error: {}", error);
                Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::INFO.message(cranelift_ir))
            }
        }
    }
}
