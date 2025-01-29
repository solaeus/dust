use std::fmt::{self, Display, Formatter};

use crate::{Type, vm::ThreadData};

use super::DustString;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AbstractList {
    pub item_type: Type,
    pub item_registers: Vec<u16>,
}

impl AbstractList {
    pub fn display(&self, data: &ThreadData) -> DustString {
        todo!()
    }
}

impl Display for AbstractList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;

        for index in &self.item_registers {
            match self.item_type {
                Type::Boolean => write!(f, "R_BOOL_{index}")?,
                Type::Byte => write!(f, "R_BYTE_{index}")?,
                Type::Character => write!(f, "R_CHAR_{index}")?,
                Type::Float => write!(f, "R_FLOAT_{index}")?,
                Type::Integer => write!(f, "R_INT_{index}")?,
                Type::String => write!(f, "R_STR_{index}")?,
                _ => todo!(),
            }
        }

        write!(f, "]")
    }
}
