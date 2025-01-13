use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct Call {
    pub destination: u16,
    pub function_register: u16,
    pub argument_count: u16,
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
        let a_field = call.destination;
        let b_field = call.function_register;
        let c_field = call.argument_count;
        let d_field = call.is_recursive;

        InstructionBuilder {
            operation: Operation::CALL,
            a_field,
            b_field,
            c_field,
            d_field,
            ..Default::default()
        }
        .build()
    }
}
