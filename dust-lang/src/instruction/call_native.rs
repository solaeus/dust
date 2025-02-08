use std::fmt::Display;

use crate::{Instruction, NativeFunction, Operation, Type};

use super::InstructionFields;

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub first_argument_index: u16,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let function = NativeFunction::from(instruction.b_field());
        let first_argument_index = instruction.c_field();

        CallNative {
            destination,
            function,
            first_argument_index,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let a_field = call_native.destination;
        let b_field = call_native.function as u16;
        let c_field = call_native.first_argument_index;

        InstructionFields {
            operation,
            a_field,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for CallNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let CallNative {
            destination,
            function,
            first_argument_index,
        } = self;
        let argument_count = function.r#type().value_parameters.len() as u16;

        if function.returns_value() {
            write!(f, "R{destination} = ")?;
        }

        write!(f, "{function}")?;

        match argument_count {
            0 => {
                write!(f, "()")
            }
            _ => {
                let arguments_end = first_argument_index + argument_count - 1;
                let arguments_index_range = *first_argument_index..=arguments_end;
                let function_type = function.r#type();
                let argument_types = function_type.value_parameters.iter();

                write!(f, "(")?;

                for (index, r#type) in arguments_index_range.zip(argument_types) {
                    match r#type {
                        Type::Boolean => {
                            write!(f, "R_BOOL_{index}")
                        }
                        Type::Byte => {
                            write!(f, "R_BYTE_{index}")
                        }
                        Type::Float => {
                            write!(f, "R_FLOAT_{index}")
                        }
                        Type::Integer => {
                            write!(f, "R_INT_{index}")
                        }
                        Type::String => {
                            write!(f, "R_STR_{index}")
                        }
                        unsupported => {
                            todo!("Support for {unsupported:?} arguments")
                        }
                    }?;

                    if index != arguments_end {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
        }
    }
}
