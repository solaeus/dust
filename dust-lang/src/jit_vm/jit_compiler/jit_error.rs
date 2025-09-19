use annotate_snippets::{Group, Level};
use cranelift_module::ModuleError;

use crate::{
    MemoryKind, OperandType, Operation, dust_error::AnnotatedError, resolver::TypeId,
    source::SourceFileId,
};

#[derive(Debug)]
pub enum JitError {
    // Cranelift errors
    CompilationError {
        message: String,
        cranelift_ir: String,
    },
    CraneliftModuleError {
        error: Box<ModuleError>,
        cranelift_ir: String,
    },

    // Missing and out-of-bounds errors
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
    UnhandledNativeFunction {
        function_name: String,
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
        arguments_start: u16,
        arguments_end: u16,
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
    TypeIndexOutOfBounds {
        type_id: TypeId,
        total_type_count: usize,
    },
    ExpectedFunctionType {
        type_id: TypeId,
    },
    MissingDeclaration {
        declaration_id: crate::resolver::DeclarationId,
    },
}

impl AnnotatedError for JitError {
    fn file_id(&self) -> SourceFileId {
        SourceFileId::default()
    }

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
            JitError::UnhandledNativeFunction { function_name } => {
                let title = format!("Unhandled native function in JIT: {}", function_name);

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
                arguments_start,
                arguments_end,
                total_argument_count,
            } => {
                let title = format!(
                    "Arguments list range {}..{} is out of bounds (total arguments: {})",
                    arguments_start, arguments_end, total_argument_count
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
            JitError::TypeIndexOutOfBounds {
                type_id,
                total_type_count,
            } => {
                let title = format!(
                    "Type ID {} out of bounds (total types: {})",
                    type_id.0, total_type_count
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::ExpectedFunctionType { type_id } => {
                let title = format!("Expected function type for Type ID {}", type_id.0);

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::MissingDeclaration { declaration_id } => {
                let title = format!(
                    "Missing declaration for Declaration ID {}",
                    declaration_id.0
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
