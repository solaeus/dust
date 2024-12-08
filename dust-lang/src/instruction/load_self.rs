use crate::{Destination, Instruction, Operation};

pub struct LoadSelf {
    pub destination: Destination,
}

impl From<&Instruction> for LoadSelf {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();

        LoadSelf { destination }
    }
}

impl From<LoadSelf> for Instruction {
    fn from(load_self: LoadSelf) -> Self {
        let (a, options) = load_self.destination.as_index_and_a_options();

        Instruction {
            operation: Operation::LOAD_SELF,
            options,
            a,
            b: 0,
            c: 0,
        }
    }
}
