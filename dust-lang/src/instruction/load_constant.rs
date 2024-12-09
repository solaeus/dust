use crate::{Instruction, Operation};

pub struct LoadConstant {
    pub destination: u8,
    pub constant_index: u8,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadConstant {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
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
        let metadata = Operation::LoadConstant as u8;
        let a = load_constant.destination;
        let b = load_constant.constant_index;
        let c = load_constant.jump_next as u8;

        Instruction { metadata, a, b, c }
    }
}
