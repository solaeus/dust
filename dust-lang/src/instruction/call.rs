use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation, r#type::TypeKind};

use super::{Address, Destination, InstructionFields, address::AddressKind};

pub struct Call {
    pub destination: Destination,
    pub function: Address,
    pub argument_list_index: u16,
    pub return_type: TypeKind,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let function_register = instruction.b_address();
        let c_address = instruction.c_address();
        let Address {
            index: argument_list_index,
            ..
        } = instruction.c_address();
        let return_type = c_address.r#type();

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
        let c_kind = match call.return_type {
            TypeKind::Boolean => AddressKind::BOOLEAN_MEMORY,
            TypeKind::Byte => AddressKind::BYTE_MEMORY,
            TypeKind::Character => AddressKind::CHARACTER_MEMORY,
            TypeKind::Float => AddressKind::FLOAT_MEMORY,
            TypeKind::Function => AddressKind::FUNCTION_MEMORY,
            TypeKind::Integer => AddressKind::INTEGER_MEMORY,
            TypeKind::List => AddressKind::LIST_MEMORY,
            TypeKind::String => AddressKind::STRING_MEMORY,
            _ => AddressKind::NONE,
        };

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
