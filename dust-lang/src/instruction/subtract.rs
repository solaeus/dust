use crate::{Argument, Instruction, Operation};

pub struct Subtract {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Subtract {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_arguments();

        Subtract {
            destination,
            left,
            right,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let operation = Operation::SUBTRACT;
        let a = subtract.destination;
        let (b, b_is_constant) = subtract.left.as_index_and_constant_flag();
        let (c, c_is_constant) = subtract.right.as_index_and_constant_flag();

        Instruction::new(operation, a, b, c, b_is_constant, c_is_constant, false)
    }
}
