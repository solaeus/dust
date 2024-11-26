use crate::{Instruction, Operation};

pub struct LoadSelf {
    pub destination: u16,
}

impl From<&Instruction> for LoadSelf {
    fn from(instruction: &Instruction) -> Self {
        LoadSelf {
            destination: instruction.a(),
        }
    }
}

impl From<LoadSelf> for Instruction {
    fn from(load_self: LoadSelf) -> Self {
        *Instruction::new(Operation::LoadSelf).set_a(load_self.destination)
    }
}
