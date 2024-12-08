use crate::{Destination, Instruction, Operation};

pub struct GetLocal {
    pub destination: Destination,
    pub local_index: u16,
}

impl From<&Instruction> for GetLocal {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();

        GetLocal {
            destination,
            local_index: instruction.b,
        }
    }
}

impl From<GetLocal> for Instruction {
    fn from(get_local: GetLocal) -> Self {
        let (a, a_options) = get_local.destination.as_index_and_a_options();

        Instruction {
            operation: Operation::GET_LOCAL,
            options: a_options,
            a,
            b: get_local.local_index,
            c: 0,
        }
    }
}
