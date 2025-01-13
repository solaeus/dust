use crate::{Instruction, Operand, Operation};

use super::InstructionBuilder;

pub struct Not {
    pub destination: u16,
    pub argument: Operand,
}

impl From<Instruction> for Not {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let argument = instruction.b_as_argument();

        Not {
            destination,
            argument,
        }
    }
}

impl From<Not> for Instruction {
    fn from(not: Not) -> Self {
        let operation = Operation::NOT;
        let a_field = not.destination;
        let (b_field, b_is_constant) = not.argument.as_index_and_constant_flag();

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            b_is_constant,
            ..Default::default()
        }
        .build()
    }
}
