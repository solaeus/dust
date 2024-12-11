use crate::{Instruction, Operation};

pub struct LoadList {
    pub destination: u8,
    pub start_register: u8,
}

impl From<&Instruction> for LoadList {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let start_register = instruction.b_field();

        LoadList {
            destination,
            start_register,
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        let operation = Operation::LOAD_LIST;
        let a = load_list.destination;
        let b = load_list.start_register;

        Instruction::new(operation, a, b, 0, false, false, false)
    }
}
