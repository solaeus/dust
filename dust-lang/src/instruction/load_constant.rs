use crate::{Destination, Instruction, Operation};

pub struct LoadConstant {
    pub destination: Destination,
    pub constant_index: u16,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadConstant {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let constant_index = instruction.b;
        let jump_next = instruction.c != 0;

        LoadConstant {
            destination,
            constant_index,
            jump_next,
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        let (a, options) = load_constant.destination.as_index_and_a_options();
        let b = load_constant.constant_index;
        let c = load_constant.jump_next as u16;

        Instruction {
            operation: Operation::LOAD_CONSTANT,
            options,
            a,
            b,
            c,
        }
    }
}
