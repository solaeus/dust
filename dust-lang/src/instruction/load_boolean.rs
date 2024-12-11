use crate::{Instruction, Operation};

pub struct LoadBoolean {
    pub destination: u8,
    pub value: bool,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadBoolean {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let value = instruction.b_field() != 0;
        let jump_next = instruction.c_field() != 0;

        LoadBoolean {
            destination,
            value,
            jump_next,
        }
    }
}

impl From<LoadBoolean> for Instruction {
    fn from(load_boolean: LoadBoolean) -> Self {
        let operation = Operation::LoadBoolean;
        let a = load_boolean.destination;
        let b = load_boolean.value as u8;
        let c = load_boolean.jump_next as u8;

        Instruction::new(operation, a, b, c, false, false, false)
    }
}
