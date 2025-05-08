use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Address, Destination, InstructionFields, TypeCode, address::AddressKind};

pub struct Call {
    pub destination: Destination,
    pub function: Address,
    pub argument_list_index: u16,
    pub return_type: TypeCode,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let function_register = instruction.b_address();
        let Address {
            index: argument_list_index,
            kind: return_kind,
        } = instruction.c_address();
        let return_type = TypeCode(return_kind.0);

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
        let c_kind = AddressKind(call.return_type.0);

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
            function: function_register,
            argument_list_index,
            return_type,
        } = self;

        destination.display(f, *return_type)?;
        write!(f, " = R_FN_{function_register}(ARGS_{argument_list_index})")
    }
}
