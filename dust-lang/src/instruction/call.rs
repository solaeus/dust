use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Address, Destination, InstructionFields, address::AddressKind};

pub struct Call {
    pub destination: Destination,
    pub function: Address,
    pub argument_list_index: u16,
    pub return_type: AddressKind,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let function_register = instruction.b_address();
        let Address {
            index: argument_list_index,
            kind: return_type,
        } = instruction.c_address();

        Call {
            destination,
            function: function_register,
            argument_list_index,
            return_type,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let operation = Operation::CALL;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = call.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = call.function;
        let c_field = call.argument_list_index;
        let c_kind = call.return_type;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            c_field,
            c_kind,
        }
        .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            function,
            argument_list_index,
            return_type,
        } = self;

        if return_type != &AddressKind::NONE {
            destination.display(f, return_type.r#type())?;
            write!(f, " = ")?;
        }

        write!(f, "{function}(ARGS_{argument_list_index})")
    }
}
