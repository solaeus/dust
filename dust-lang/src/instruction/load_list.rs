use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, TypeCode};

pub struct LoadList {
    pub destination: u16,
    pub item_type: TypeCode,
    pub start_register: u16,
    pub length: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadList {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let start_register = instruction.b_field();
        let item_type = instruction.b_type();
        let length = instruction.c_field();
        let jump_next = instruction.d_field();

        LoadList {
            destination,
            item_type,
            start_register,
            length,
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
            c_field: load_list.length,
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
            length,
            jump_next,
        } = self;
        let type_caps = item_type.to_string().to_uppercase();

        write!(f, "R_LIST_{destination} = [")?;

        for (i, register_index) in (*start_register..(start_register + length)).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "R_{type_caps}_{register_index}")?;
        }

        write!(f, "]")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
