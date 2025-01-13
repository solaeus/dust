use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation, TypeCode};

pub struct Modulo {
    pub destination: u16,
    pub left: Operand,
    pub left_type: TypeCode,
    pub right: Operand,
    pub right_type: TypeCode,
}

impl From<Instruction> for Modulo {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();
        let left_type = instruction.b_type();
        let right_type = instruction.c_type();

        Modulo {
            destination,
            left,
            left_type,
            right,
            right_type,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let operation = Operation::MODULO;
        let a_field = modulo.destination;
        let (b_field, b_is_constant) = modulo.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = modulo.right.as_index_and_constant_flag();
        let b_type = modulo.left_type;
        let c_type = modulo.right_type;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            c_field,
            b_is_constant,
            c_is_constant,
            b_type,
            c_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Modulo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Modulo {
            destination,
            left,
            left_type,
            right,
            right_type,
        } = self;

        write!(
            f,
            "R{destination} = {left_type}({left}) % {right_type}({right})",
        )
    }
}
