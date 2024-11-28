use crate::{Destination, Instruction, Operation};

pub struct LoadList {
    pub destination: Destination,
    pub start_register: u16,
}

impl From<&Instruction> for LoadList {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        LoadList {
            destination,
            start_register: instruction.b(),
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        let (a, a_is_local) = match load_list.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::LoadList)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(load_list.start_register)
    }
}
