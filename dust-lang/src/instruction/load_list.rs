use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Address, AddressKind, Destination, InstructionFields, TypeKind};

pub struct LoadList {
    pub destination: Destination,
    pub start: Address,
    pub end: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadList {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let start_register = instruction.b_address();
        let (end_register, jump_next) = {
            let Address { index, kind } = instruction.c_address();
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
        let Address {
            index: b_field,
            kind: b_kind,
        } = load_list.start;
        let c_field = load_list.end;
        let c_kind = {
            let jump_next_encoded = load_list.jump_next as u8;

            AddressKind(jump_next_encoded)
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
            start,
            end: end_index,
            jump_next,
        } = self;
        let end = Address::new(*end_index, start.kind);

        if destination.is_register {
            write!(f, "R_LIST_{} = [", destination.index)?;
        } else {
            write!(f, "M_LIST_{} = [", destination.index)?;
        }

        match start.r#type() {
            TypeKind::Boolean => {
                write!(f, "{start}..={end}")?;
            }
            TypeKind::Byte => {
                write!(f, "{start}..={end}")?;
            }
            TypeKind::Character => {
                write!(f, "{start}..={end}")?;
            }
            TypeKind::Float => {
                write!(f, "{start}..={end}")?;
            }
            TypeKind::Integer => {
                write!(f, "{start}..={end}")?;
            }
            TypeKind::String => {
                write!(f, "{start}..={end}")?;
            }
            TypeKind::List => {
                write!(f, "{start}..={end}")?;
            }
            TypeKind::Function => {
                write!(f, "{start}..={end}")?;
            }
            invalid => invalid.write_invalid(f)?,
        }

        write!(f, "]")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
