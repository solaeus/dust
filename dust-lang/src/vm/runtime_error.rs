use crate::{AnnotatedError, Span};

pub const RUNTIME_ERROR_TEXT: &str = "An error occurred during the execution of the Dust VM. This is a bug in the VM or the compiler.";

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum RuntimeError {
    // Call stack errors
    CallStackUnderflow,

    // Register stack errors
    RegisterIndexOutOfBounds,

    // Object stack errors
    ObjectIndexOutOfBounds,
    InvalidObjectType,

    // Constants errors
    InvalidConstantIndex,
    InvalidConstantType,

    // Instruction errors
    InvalidOperation,
    InvalidOperandType,
    InvalidMemoryKind,

    // User errors
    DivisionByZero,
}

impl AnnotatedError for RuntimeError {
    fn title(&self) -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        match self {
            RuntimeError::CallStackUnderflow => "Call stack underflow",
            RuntimeError::RegisterIndexOutOfBounds => "Register index out of bounds",
            RuntimeError::ObjectIndexOutOfBounds => "Object index out of bounds",
            RuntimeError::InvalidObjectType => "Invalid object type",
            RuntimeError::InvalidConstantIndex => "Invalid constant index",
            RuntimeError::InvalidConstantType => "Invalid constant type",
            RuntimeError::InvalidOperation => "Invalid operation",
            RuntimeError::InvalidOperandType => "Invalid operand type",
            RuntimeError::InvalidMemoryKind => "Invalid memory kind",
            RuntimeError::DivisionByZero => "Division by zero",
        }
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        vec![]
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        vec![]
    }
}
