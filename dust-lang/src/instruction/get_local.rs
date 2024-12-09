use crate::{Instruction, Operation};

pub struct GetLocal {
    pub destination: u8,
    pub local_index: u8,
}

impl From<&Instruction> for GetLocal {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let local_index = instruction.b;

        GetLocal {
            destination,
            local_index,
        }
    }
}

impl From<GetLocal> for Instruction {
    fn from(get_local: GetLocal) -> Self {
        let a = get_local.destination;
        let b = get_local.local_index;
        let c = 0;
        let metadata = Operation::GetLocal as u8;

        Instruction { metadata, a, b, c }
    }
}
