use crate::{Destination, Instruction, Operation};

pub struct LoadBoolean {
    pub destination: Destination,
    pub value: bool,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadBoolean {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        LoadBoolean {
            destination,
            value: instruction.b_as_boolean(),
            jump_next: instruction.c_as_boolean(),
        }
    }
}

impl From<LoadBoolean> for Instruction {
    fn from(load_boolean: LoadBoolean) -> Self {
        let (a, a_is_local) = match load_boolean.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::LoadBoolean)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b_to_boolean(load_boolean.value)
            .set_c_to_boolean(load_boolean.jump_next)
    }
}
