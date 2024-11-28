use crate::{Destination, Instruction, Operation};

pub struct LoadSelf {
    pub destination: Destination,
}

impl From<&Instruction> for LoadSelf {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        LoadSelf { destination }
    }
}

impl From<LoadSelf> for Instruction {
    fn from(load_self: LoadSelf) -> Self {
        let (a, a_is_local) = match load_self.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::LoadSelf)
            .set_a(a)
            .set_a_is_local(a_is_local)
    }
}
