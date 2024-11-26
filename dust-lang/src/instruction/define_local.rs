use crate::{Instruction, Operation};

pub struct DefineLocal {
    pub register: u16,
    pub local_index: u16,
    pub is_mutable: bool,
}

impl From<&Instruction> for DefineLocal {
    fn from(instruction: &Instruction) -> Self {
        DefineLocal {
            register: instruction.a(),
            local_index: instruction.b(),
            is_mutable: instruction.c_as_boolean(),
        }
    }
}

impl From<DefineLocal> for Instruction {
    fn from(define_local: DefineLocal) -> Self {
        *Instruction::new(Operation::DefineLocal)
            .set_a(define_local.register)
            .set_b(define_local.local_index)
            .set_c_to_boolean(define_local.is_mutable)
    }
}
