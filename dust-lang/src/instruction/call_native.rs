use std::fmt::{self, Display, Formatter};

use crate::{instruction::OperandType, native_function::NativeFunction};

use super::{Instruction, InstructionFields, Operation};

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub arguments_start: u16,
    pub return_type: OperandType,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let function = NativeFunction::from_index(instruction.b_field());
        let arguments_start = instruction.c_field();
        let return_type = instruction.operand_type();

        CallNative {
            destination,
            function,
            arguments_start,
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
        let operand_type = call_native.return_type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            c_field,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for CallNative {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let CallNative {
            destination,
            function,
            arguments_start,
            return_type,
        } = self;
        let argument_count = function.argument_count();

        if *return_type != OperandType::NONE {
            write!(f, "reg_{destination} = ")?;
        }

        if argument_count == 0 {
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
