use crate::{Destination, Instruction, Operation};

pub struct GetLocal {
    pub destination: Destination,
    pub local_index: u16,
}

impl From<&Instruction> for GetLocal {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        GetLocal {
            destination,
            local_index: instruction.b(),
        }
    }
}

impl From<GetLocal> for Instruction {
    fn from(get_local: GetLocal) -> Self {
        let (a, a_is_local) = match get_local.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::GetLocal)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(get_local.local_index)
    }
}
