use crate::{Instruction, Operation};

pub struct GetLocal {
    pub destination: u8,
    pub local_index: u8,
}

impl From<&Instruction> for GetLocal {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let local_index = instruction.b_field();

        GetLocal {
            destination,
            local_index,
        }
    }
}

impl From<GetLocal> for Instruction {
    fn from(get_local: GetLocal) -> Self {
        let operation = Operation::GetLocal;
        let a = get_local.destination;
        let b = get_local.local_index;

        Instruction::new(operation, a, b, 0, false, false, false)
    }
}
