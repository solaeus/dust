use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct ModuloInt {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for ModuloInt {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        ModuloInt {
            destination,
            left,
            right,
        }
    }
}

impl From<ModuloInt> for Instruction {
    fn from(modulo_int: ModuloInt) -> Self {
        let operation = Operation::MODULO_INT;
        let a_field = modulo_int.destination;
        let (b_field, b_is_constant) = modulo_int.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = modulo_int.right.as_index_and_constant_flag();

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

impl Display for ModuloInt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ModuloInt {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} % {}", destination, left, right)
    }
}
