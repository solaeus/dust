use std::fmt::Display;

use crate::{NativeFunction, OperandType};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct CallNative<C> {
    pub destination: Address,
    pub function: NativeFunction<C>,
    pub argument_list_index: u16,
}

impl<C> From<&Instruction> for CallNative<C> {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let function = NativeFunction::from_index(instruction.b_field());
        let argument_list_index = instruction.c_field();

        CallNative {
            destination,
            function,
            argument_list_index,
        }
    }
}

impl<C> From<CallNative<C>> for Instruction {
    fn from(call_native: CallNative<C>) -> Self {
        let operation = Operation::CALL_NATIVE;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = call_native.destination;
        let b_field = call_native.function.index;
        let c_field = call_native.argument_list_index;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl<C> Display for CallNative<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let CallNative {
            destination,
            function,
            argument_list_index,
        } = self;
        let return_type = function.r#type().return_type.as_operand_type();

        if return_type != OperandType::NONE {
            destination.display(f, return_type)?;
            write!(f, " = ")?;
        }

        write!(f, "{function}(ARGS_{argument_list_index})")
    }
}
