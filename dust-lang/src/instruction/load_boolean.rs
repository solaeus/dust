use crate::{Destination, Instruction, Operation};

pub struct LoadBoolean {
    pub destination: Destination,
    pub value: bool,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadBoolean {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let value = instruction.b != 0;
        let jump_next = instruction.c != 0;

        LoadBoolean {
            destination,
            value,
            jump_next,
        }
    }
}

impl From<LoadBoolean> for Instruction {
    fn from(load_boolean: LoadBoolean) -> Self {
        let (a, options) = load_boolean.destination.as_index_and_a_options();
        let b = load_boolean.value as u16;
        let c = load_boolean.jump_next as u16;

        Instruction {
            operation: Operation::LOAD_BOOLEAN,
            options,
            a,
            b,
            c,
        }
    }
}
