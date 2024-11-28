use crate::{Destination, Instruction, Operation};

pub struct LoadConstant {
    pub destination: Destination,
    pub constant_index: u16,
    pub jump_next: bool,
}

impl From<&Instruction> for LoadConstant {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        LoadConstant {
            destination,
            constant_index: instruction.b(),
            jump_next: instruction.c_as_boolean(),
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        let (a, a_is_local) = match load_constant.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::LoadConstant)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(load_constant.constant_index)
            .set_c_to_boolean(load_constant.jump_next)
    }
}
