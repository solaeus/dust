use crate::{Instruction, Operation};

pub struct LoadList {
    pub destination: u16,
    pub start_register: u16,
}

impl From<&Instruction> for LoadList {
    fn from(instruction: &Instruction) -> Self {
        LoadList {
            destination: instruction.a(),
            start_register: instruction.b(),
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        *Instruction::new(Operation::LoadList)
            .set_a(load_list.destination)
            .set_b(load_list.start_register)
    }
}
