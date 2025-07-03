use crate::{Address, AnnotatedError, OperandType, Span};

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    InvalidAddress(Address),
    InvalidOperandType(OperandType),
    InvalidObject(Address),
}

impl AnnotatedError for RuntimeError {
    fn title() -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        match self {
            RuntimeError::InvalidAddress(_) => "Invalid address",
            RuntimeError::InvalidOperandType(_) => "Invalid operand type in address",
            RuntimeError::InvalidObject(_) => "Invalid object at address",
        }
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        match self {
            RuntimeError::InvalidAddress(address) => {
                vec![(
                    format!(
                        "This address is malformed or invalid for the current operation: {address}"
                    ),
                    Span(0, 0),
                )]
            }
            RuntimeError::InvalidOperandType(operand_type) => {
                vec![(
                    format!(
                        "The operand type is malformed or invalid for the current operation: {operand_type}"
                    ),
                    Span(0, 0),
                )]
            }
            RuntimeError::InvalidObject(address) => {
                vec![(
                    format!("The object at address {address} is invalid or not found"),
                    Span(0, 0),
                )]
            }
        }
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        vec![(
            "The Dust chunk passed to the VM contains invalid data due to corruption or an error in the compiler."
                .to_string(),
            Span(0, 0),
        )]
    }
}
