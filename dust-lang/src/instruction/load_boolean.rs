use crate::{Instruction, Operation};

pub struct LoadBoolean {
    pub destination: u16,
    pub value: bool,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadBoolean {
    fn from(instruction: &Instruction) -> Self {
        LoadBoolean {
            destination: instruction.a(),
            value: instruction.b_as_boolean(),
            jump_next: instruction.c_as_boolean(),
        }
    }
}

impl From<LoadBoolean> for Instruction {
    fn from(load_boolean: LoadBoolean) -> Self {
        *Instruction::new(Operation::LoadBoolean)
            .set_a(load_boolean.destination)
            .set_b_to_boolean(load_boolean.value)
            .set_c_to_boolean(load_boolean.jump_next)
    }
}
