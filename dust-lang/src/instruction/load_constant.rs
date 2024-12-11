use crate::{Instruction, Operation};

pub struct LoadConstant {
    pub destination: u8,
    pub constant_index: u8,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadConstant {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let constant_index = instruction.b_field();
        let jump_next = instruction.c_field() != 0;

        LoadConstant {
            destination,
            constant_index,
            jump_next,
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        let operation = Operation::LOAD_CONSTANT;
        let a = load_constant.destination;
        let b = load_constant.constant_index;
        let c = load_constant.jump_next as u8;

        Instruction::new(operation, a, b, c, false, false, false)
    }
}
