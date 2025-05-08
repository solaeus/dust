use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Destination, InstructionFields, Operand, OperandKind, TypeCode};

pub struct LoadList {
    pub destination: Destination,
    pub start: Operand,
    pub end: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadList {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let start_register = instruction.b_operand();
        let (end_register, jump_next) = {
            let Operand { index, kind } = instruction.c_operand();
            let jump_next = kind.0 != 0;

            (index, jump_next)
        };

        LoadList {
            destination,
            start: start_register,
            end: end_register,
            jump_next,
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        let operation = Operation::LOAD_LIST;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = load_list.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = load_list.start;
        let c_field = load_list.end;
        let c_kind = {
            let jump_next_encoded = load_list.jump_next as u8;

            OperandKind(jump_next_encoded)
        };

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            c_field,
            c_kind,
        }
        .build()
    }
}

impl Display for LoadList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadList {
            destination,
            start: start_register,
            end: end_register,
            jump_next,
        } = self;

        write!(f, "{} = [", destination.index)?;

        match start_register.as_type_code() {
            TypeCode::BOOLEAN => {
                write!(f, "{start_register}..=R_BOOL_{end_register}")?;
            }
            TypeCode::BYTE => {
                write!(f, "{start_register}..=R_BYTE_{end_register}")?;
            }
            TypeCode::CHARACTER => {
                write!(f, "{start_register}..=R_CHAR_{end_register}")?;
            }
            TypeCode::FLOAT => {
                write!(f, "{start_register}..=R_FLOAT_{end_register}")?;
            }
            TypeCode::INTEGER => {
                write!(f, "{start_register}..=R_INT_{end_register}")?;
            }
            TypeCode::STRING => {
                write!(f, "{start_register}..=R_STR_{end_register}")?;
            }
            TypeCode::LIST => {
                write!(f, "{start_register}..=R_LIST_{end_register}")?;
            }
            TypeCode::FUNCTION => {
                write!(f, "{start_register}..=R_FN_{end_register}")?;
            }
            unsupported => unsupported.unsupported_write(f)?,
        }

        write!(f, "]")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
