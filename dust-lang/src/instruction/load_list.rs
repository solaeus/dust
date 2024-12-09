use crate::{Instruction, Operation};

pub struct LoadList {
    pub destination: u8,
    pub start_register: u8,
}

impl From<&Instruction> for LoadList {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let start_register = instruction.b;

        LoadList {
            destination,
            start_register,
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        let metadata = Operation::LoadList as u8;
        let a = load_list.destination;
        let b = load_list.start_register;
        let c = 0;

        Instruction { metadata, a, b, c }
    }
}
