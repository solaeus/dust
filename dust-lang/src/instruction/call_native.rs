use std::fmt::Display;

use crate::{Instruction, NativeFunction, Operation, Type};

use super::ZeroOperandLayout;

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub first_argument: u16,
    pub argument_count: u8,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let function = NativeFunction::from(instruction.b_field());
        let first_argument = instruction.c_field();
        let argument_count = instruction.e_field();

        CallNative {
            destination,
            function,
            first_argument,
            argument_count,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let a_field = call_native.destination;
        let b_field = call_native.function as u16;
        let c_field = call_native.first_argument;
        let e_field = call_native.argument_count;

        ZeroOperandLayout {
            operation,
            a_field,
            b_field,
            c_field,
            e_field,
        }
        .build()
    }
}

impl Display for CallNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let CallNative {
            destination,
            function,
            first_argument,
            argument_count,
        } = self;
        let function_type = function.r#type();

        if function.returns_value() {
            match function_type.return_type {
                Type::Boolean => write!(f, "R_BOOL_{destination} = ")?,
                Type::Byte => write!(f, "R_BYTE_{destination} = ")?,
                Type::Character => write!(f, "R_CHAR_{destination} = ")?,
                Type::Float => write!(f, "R_FLOAT_{destination} = ")?,
                Type::Integer => write!(f, "R_INT_{destination} = ")?,
                Type::String => write!(f, "R_STR_{destination} = ")?,
                Type::None => {}
                _ => todo!(),
            }
        }

        if *argument_count == 0 {
            write!(f, "{function}()")
        } else {
            let last_argument = first_argument + (*argument_count as u16).saturating_sub(1);
            let arguments = *first_argument..=last_argument;
            let parameter_types = function_type
                .value_parameters
                .iter()
                .map(|(_, r#type)| r#type);

            write!(f, "{function}(")?;

            let mut is_first = true;

            for (argument, parameter_type) in arguments.zip(parameter_types) {
                if is_first {
                    is_first = false;
                } else {
                    write!(f, ", ")?;
                }

                match parameter_type {
                    Type::Boolean => write!(f, "R_BOOL_{argument}")?,
                    Type::Byte => write!(f, "R_BYTE_{argument}")?,
                    Type::Character => write!(f, "R_CHAR_{argument}")?,
                    Type::Float => write!(f, "R_FLOAT_{argument}")?,
                    Type::Integer => write!(f, "R_INT_{argument}")?,
                    Type::String => write!(f, "R_STR_{argument}")?,
                    Type::None => {}
                    _ => todo!(),
                }
            }

            write!(f, ")")
        }
    }
}
