use crate::{Destination, Instruction, Operation};

pub struct LoadList {
    pub destination: Destination,
    pub start_register: u16,
}

impl From<&Instruction> for LoadList {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let start_register = instruction.b;

        LoadList {
            destination,
            start_register,
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        let (a, options) = load_list.destination.as_index_and_a_options();
        let b = load_list.start_register;

        Instruction {
            operation: Operation::LOAD_LIST,
            options,
            a,
            b,
            c: 0,
        }
    }
}
