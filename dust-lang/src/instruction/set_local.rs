use crate::{Instruction, Operation};

pub struct SetLocal {
    pub register: u16,
    pub local_index: u16,
}

impl From<&Instruction> for SetLocal {
    fn from(instruction: &Instruction) -> Self {
        SetLocal {
            register: instruction.a(),
            local_index: instruction.b(),
        }
    }
}

impl From<SetLocal> for Instruction {
    fn from(set_local: SetLocal) -> Self {
        *Instruction::new(Operation::SetLocal)
            .set_a(set_local.register)
            .set_b(set_local.local_index)
    }
}
