use crate::{Instruction, Operation};

pub struct LoadConstant {
    pub destination: u16,
    pub constant_index: u16,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadConstant {
    fn from(instruction: &Instruction) -> Self {
        LoadConstant {
            destination: instruction.a(),
            constant_index: instruction.b(),
            jump_next: instruction.c_as_boolean(),
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        *Instruction::new(Operation::LoadConstant)
            .set_a(load_constant.destination)
            .set_b(load_constant.constant_index)
            .set_c_to_boolean(load_constant.jump_next)
    }
}
