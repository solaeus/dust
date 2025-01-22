use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{TwoOperandLayout, TypeCode};

pub struct GetLocal {
    pub destination: u16,
    pub local_index: u16,
    pub r#type: TypeCode,
}

impl From<Instruction> for GetLocal {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let local_index = instruction.b_field();
        let r#type = instruction.b_type();

        GetLocal {
            destination,
            local_index,
            r#type,
        }
    }
}

impl From<GetLocal> for Instruction {
    fn from(get_local: GetLocal) -> Self {
        let operation = Operation::GET_LOCAL;
        let a_field = get_local.destination;
        let b_field = get_local.local_index;
        let b_type = get_local.r#type;

        TwoOperandLayout {
            operation,
            a_field,
            b_field,
            b_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for GetLocal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let GetLocal {
            destination,
            local_index,
            r#type,
        } = self;
        let register_name = r#type.register_name();

        write!(f, "{register_name}_{destination} = L{local_index}")
    }
}
