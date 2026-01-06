use annotate_snippets::{Group, Level};
use cranelift_module::ModuleError;

use crate::{
    dust_error::AnnotatedError,
    instruction::{MemoryKind, OperandType, Operation},
    resolver::TypeId,
    source::SourceFileId,
    r#type::Type,
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
        cranelift_ir: Option<String>,
    },

    // Missing and out-of-bounds errors
    UnsupportedOperandType {
        operand_type: OperandType,
    },
    DropListIndexOutOfBounds {
        drop_list_index: u16,

        drop_list_length: usize,
    },
    UnsupportedOperation {
        operation: Operation,
    },
    UnsupportedNativeFunction {
        function_name: &'static str,
    },
    UnsupportedMemoryKind {
        memory_kind: MemoryKind,
    },
    FunctionIndexOutOfBounds {
        ip: usize,
        function_index: u16,
        total_function_count: usize,
    },
    CallArgumentIndexOutOfBounds {
        argument_index: usize,
        total_argument_count: usize,
    },
    ConstantIndexOutOfBounds {
        constant_index: u16,
        total_constant_count: usize,
    },
    InvalidConstantType {
        expected_type: OperandType,
    },
    InvalidObjectType {
        expected: Type,
    },
    InvalidObjectValue {
        expected: OperandType,
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
    MissingReturnValue,

    // Execution errors
    ThreadErrorFunctionIndexOutOfBounds,
    ThreadErrorListIndexOutOfBounds,
    ThreadErrorDivisionByZero,
    MissingPrototype {
        index: usize,
        total: usize,
    },
    InstructionIndexOutOfBounds {
        instruction_index: usize,
        total_instruction_count: usize,
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
            JitError::UnsupportedOperation { operation } => {
                let title = format!("Unsupported operation in JIT: {:?}", operation);

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::UnsupportedNativeFunction { function_name } => {
                let title = format!("Unsupported native function in JIT: {}", function_name);

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::UnsupportedMemoryKind { memory_kind } => {
                let title = format!("Unsupported memory kind in JIT: {:?}", memory_kind);

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::DropListIndexOutOfBounds {
                drop_list_index,
                drop_list_length,
            } => {
                let title = format!(
                    "Drop list index {drop_list_index} is out of bounds (drop list length: {drop_list_length})"
                );

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
            JitError::CallArgumentIndexOutOfBounds {
                argument_index,
                total_argument_count,
            } => {
                let title = format!(
                    "Call argument index {} out of bounds (total arguments: {})",
                    argument_index, total_argument_count
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
            JitError::InvalidObjectType { expected } => {
                let title = format!("Invalid object type; expected {:?}", expected);

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::InvalidObjectValue { expected } => {
                let title = format!("Invalid object value; expected {:?}", expected);

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
                let info = match cranelift_ir {
                    Some(ir) => ir,
                    None => "<no Cranelift IR available>",
                };

                Group::with_title(Level::ERROR.primary_title(title))
                    .element(Level::INFO.message(info))
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
            JitError::MissingPrototype { index, total } => {
                let title = format!(
                    "Missing prototype at index {} (total prototypes: {})",
                    index, total
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::ThreadErrorFunctionIndexOutOfBounds => {
                let title = "Function index out of bounds".to_string();

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::ThreadErrorListIndexOutOfBounds => {
                let title = "List index out of bounds".to_string();

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::ThreadErrorDivisionByZero => {
                let title = "Division by zero".to_string();

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::InstructionIndexOutOfBounds {
                instruction_index,
                total_instruction_count,
            } => {
                let title = format!(
                    "Instruction index {} out of bounds (total instructions: {})",
                    instruction_index, total_instruction_count
                );

                Group::with_title(Level::ERROR.primary_title(title))
            }
            JitError::MissingReturnValue => {
                let title = "Missing return value from function".to_string();

                Group::with_title(Level::ERROR.primary_title(title))
            }
        }
    }
}
