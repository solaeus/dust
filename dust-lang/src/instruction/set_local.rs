use crate::{Instruction, Operation};

pub struct SetLocal {
    pub register_index: u8,
    pub local_index: u8,
}

impl From<&Instruction> for SetLocal {
    fn from(instruction: &Instruction) -> Self {
        let register_index = instruction.b;
        let local_index = instruction.c;

        SetLocal {
            register_index,
            local_index,
        }
    }
}

impl From<SetLocal> for Instruction {
    fn from(set_local: SetLocal) -> Self {
        let metadata = Operation::SetLocal as u8;
        let a = 0;
        let b = set_local.register_index;
        let c = set_local.local_index;

        Instruction { metadata, a, b, c }
    }
}
