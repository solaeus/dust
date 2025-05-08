use std::fmt::{self, Display, Formatter};

use tracing::error;

use super::{
    Address, AddressKind, Destination, Instruction, InstructionFields, Operation, TypeCode,
};

pub struct LoadFunction {
    pub destination: Destination,
    pub prototype: Address,
    pub jump_next: bool,
}

impl From<Instruction> for LoadFunction {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let prototype = instruction.b_address();
        let jump_next = instruction.c_field() != 0;

        LoadFunction {
            destination,
            prototype,
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
        let Address {
            index: b_field,
            kind: b_kind,
        } = load_function.prototype;
        let c_field = load_function.jump_next as u16;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
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
            prototype,
            jump_next,
        } = self;

        destination.display(f, TypeCode::FUNCTION)?;

        match prototype.kind {
            AddressKind::FUNCTION_PROTOTYPE => write!(f, " = PROTO_{}", prototype.index)?,
            AddressKind::FUNCTION_SELF => write!(f, " = SELF")?,
            _ => {
                error!("Invalid memory address: {prototype}");
                write!(f, " = INVALID")?
            }
        }

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
