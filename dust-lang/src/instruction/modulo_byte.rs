use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct ModuloByte {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for ModuloByte {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        ModuloByte {
            destination,
            left,
            right,
        }
    }
}

impl From<ModuloByte> for Instruction {
    fn from(modulo_byte: ModuloByte) -> Self {
        let operation = Operation::MODULO_BYTE;
        let a_field = modulo_byte.destination;
        let (b_field, b_is_constant) = modulo_byte.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = modulo_byte.right.as_index_and_constant_flag();

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

impl Display for ModuloByte {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ModuloByte {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} % {}", destination, left, right)
    }
}
