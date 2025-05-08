use std::fmt::{self, Display, Formatter};

use super::{Destination, Instruction, InstructionFields, Operation, TypeCode};

pub struct LoadFunction {
    pub destination: Destination,
    pub prototype_index: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadFunction {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let prototype_index = instruction.b_field();
        let jump_next = instruction.c_field() != 0;

        LoadFunction {
            destination,
            prototype_index,
            jump_next,
        }
    }
}

impl From<LoadFunction> for Instruction {
    fn from(load_function: LoadFunction) -> Self {
        let operation = Operation::LOAD_FUNCTION;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = load_function.destination;
        let b_field = load_function.prototype_index;
        let c_field = load_function.jump_next as u16;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadFunction {
            destination,
            prototype_index,
            jump_next,
        } = self;

        destination.display(f, TypeCode::FUNCTION)?;
        write!(f, " = PROTO_{prototype_index}")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
