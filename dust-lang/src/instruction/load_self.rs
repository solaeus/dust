use crate::{Instruction, Operation};

pub struct LoadSelf {
    pub destination: u8,
}

impl From<&Instruction> for LoadSelf {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();

        LoadSelf { destination }
    }
}

impl From<LoadSelf> for Instruction {
    fn from(load_self: LoadSelf) -> Self {
        let operation = Operation::LoadSelf;
        let a = load_self.destination;

        Instruction::new(operation, a, 0, 0, false, false, false)
    }
}
