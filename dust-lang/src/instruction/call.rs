use crate::{Instruction, Operation};

pub struct Call {
    pub destination: u8,
    pub function_register: u8,
    pub argument_count: u8,
    pub is_recursive: bool,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let function_register = instruction.b_field();
        let argument_count = instruction.c_field();
        let is_recursive = instruction.d_field();

        Call {
            destination,
            function_register,
            argument_count,
            is_recursive,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let a = call.destination;
        let b = call.function_register;
        let c = call.argument_count;
        let d = call.is_recursive;

        Instruction::new(Operation::CALL, a, b, c, false, false, d)
    }
}
