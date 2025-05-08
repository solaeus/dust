use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Destination, InstructionFields, Operand, TypeCode};

pub struct LoadConstant {
    pub destination: Destination,
    pub constant: Operand,
    pub jump_next: bool,
}

impl From<Instruction> for LoadConstant {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let constant = instruction.b_operand();
        let jump_next = instruction.c_field() != 0;

        LoadConstant {
            destination,
            constant,
            jump_next,
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        let operation = Operation::LOAD_CONSTANT;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = load_constant.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = load_constant.constant;
        let c_field = load_constant.jump_next as u16;

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

impl Display for LoadConstant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadConstant {
            destination:
                Destination {
                    index: destination_index,
                    ..
                },
            constant,
            jump_next,
        } = self;

        match constant.as_type_code() {
            TypeCode::CHARACTER => write!(f, "R_CHAR_{destination_index}")?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination_index}")?,
            TypeCode::INTEGER => write!(f, "R_INT_{destination_index}")?,
            TypeCode::STRING => write!(f, "R_STR_{destination_index}")?,
            unsupported => unsupported.unsupported_write(f)?,
        }

        write!(f, " = {constant}");

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
