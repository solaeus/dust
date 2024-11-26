use crate::{Instruction, Operation};

pub struct GetLocal {
    pub destination: u16,
    pub local_index: u16,
}

impl From<&Instruction> for GetLocal {
    fn from(instruction: &Instruction) -> Self {
        GetLocal {
            destination: instruction.a(),
            local_index: instruction.b(),
        }
    }
}

impl From<GetLocal> for Instruction {
    fn from(get_local: GetLocal) -> Self {
        *Instruction::new(Operation::GetLocal)
            .set_a(get_local.destination)
            .set_b(get_local.local_index)
    }
}
