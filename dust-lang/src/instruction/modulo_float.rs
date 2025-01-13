use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct ModuloFloat {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for ModuloFloat {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        ModuloFloat {
            destination,
            left,
            right,
        }
    }
}

impl From<ModuloFloat> for Instruction {
    fn from(modulo_float: ModuloFloat) -> Self {
        let operation = Operation::MODULO_FLOAT;
        let a_field = modulo_float.destination;
        let (b_field, b_is_constant) = modulo_float.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = modulo_float.right.as_index_and_constant_flag();

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            c_field,
            b_is_constant,
            c_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for ModuloFloat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ModuloFloat {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} % {}", destination, left, right)
    }
}
