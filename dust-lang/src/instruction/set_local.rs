use crate::{Instruction, Operation};

use super::InstructionOptions;

pub struct SetLocal {
    pub register_index: u16,
    pub local_index: u16,
}

impl From<&Instruction> for SetLocal {
    fn from(instruction: &Instruction) -> Self {
        let register_index = instruction.b;
        let local_index = instruction.c;

        SetLocal {
            register_index,
            local_index,
        }
    }
}

impl From<SetLocal> for Instruction {
    fn from(set_local: SetLocal) -> Self {
        let b = set_local.register_index;
        let c = set_local.local_index;

        Instruction {
            operation: Operation::SET_LOCAL,
            options: InstructionOptions::empty(),
            a: 0,
            b,
            c,
        }
    }
}
