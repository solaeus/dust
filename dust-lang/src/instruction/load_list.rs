use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, TypeCode};

pub struct LoadList {
    pub destination: u16,
    pub item_type: TypeCode,
    pub start_register: u16,
    pub end_register: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadList {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let start_register = instruction.b_field();
        let item_type = instruction.b_type();
        let end_register = instruction.c_field();
        let jump_next = instruction.d_field();

        LoadList {
            destination,
            item_type,
            start_register,
            end_register,
            jump_next,
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        InstructionFields {
            operation: Operation::LOAD_LIST,
            a_field: load_list.destination,
            b_field: load_list.start_register,
            b_type: load_list.item_type,
            c_field: load_list.end_register,
            d_field: load_list.jump_next,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadList {
            destination,
            item_type,
            start_register,
            end_register,
            jump_next,
        } = self;

        write!(f, "R_LIST_{destination} = [")?;

        match *item_type {
            TypeCode::BOOLEAN => {
                write!(f, "R_BOOL_{start_register}..=R_BOOL_{end_register}")?;
            }
            TypeCode::BYTE => {
                write!(f, "R_BYTE_{start_register}..=R_BYTE_{end_register}")?;
            }
            TypeCode::CHARACTER => {
                write!(f, "R_CHAR_{start_register}..=R_CHAR_{end_register}")?;
            }
            TypeCode::FLOAT => {
                write!(f, "R_FLOAT_{start_register}..=R_FLOAT_{end_register}")?;
            }
            TypeCode::INTEGER => {
                write!(f, "R_INT_{start_register}..=R_INT_{end_register}")?;
            }
            TypeCode::STRING => {
                write!(f, "R_STR_{start_register}..=R_STR_{end_register}")?;
            }
            TypeCode::LIST => {
                write!(f, "R_LIST_{start_register}..=R_LIST_{end_register}")?;
            }
            TypeCode::FUNCTION => {
                write!(f, "R_FN_{start_register}..=R_FN_{end_register}")?;
            }
            unknown => panic!("Unknown type code: {}", unknown.0),
        }

        write!(f, "]")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
