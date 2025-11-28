use std::fmt::Display;

use crate::{instruction::OperandType, native_function::NativeFunction};

use super::{Instruction, InstructionFields, Operation};

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub arguments_start: u16,
    pub argument_count: u16,
    pub return_type: OperandType,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let function = NativeFunction::from_index(instruction.b_field());
        let arguments_start = instruction.c_field();
        let function_type = function.r#type();
        let argument_count = function_type.value_parameters.len() as u16;
        let return_type = function_type.return_type.as_operand_type();

        CallNative {
            destination,
            function,
            arguments_start,
            argument_count,
            return_type,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let a_field = call_native.destination;
        let b_field = call_native.function.index;
        let c_field = call_native.arguments_start;
        let d_field = Some(call_native.argument_count);
        let operand_type = call_native.return_type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            c_field,
            d_field,
            operand_type,
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
            arguments_start,
            argument_count,
            ..
        } = self;

        if function.returns_value() {
            write!(f, "reg_{destination} = ")?;
        }

        if *argument_count == 0 {
            write!(f, "{function}()")
        } else {
            let arguments_end = arguments_start + argument_count;

            write!(
                f,
                "{function}(args_{arguments_start}..args_{arguments_end})"
            )
        }
    }
}
