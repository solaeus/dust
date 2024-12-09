use crate::{Instruction, Operation};

pub struct LoadSelf {
    pub destination: u8,
}

impl From<&Instruction> for LoadSelf {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;

        LoadSelf { destination }
    }
}

impl From<LoadSelf> for Instruction {
    fn from(load_self: LoadSelf) -> Self {
        let metadata = Operation::LoadSelf as u8;
        let a = load_self.destination;
        let b = 0;
        let c = 0;

        Instruction { metadata, a, b, c }
    }
}
