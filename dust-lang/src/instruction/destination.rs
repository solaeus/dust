use std::fmt::{self, Formatter};

use crate::r#type::TypeKind;

use super::{Address, AddressKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Destination {
    pub index: u16,
    pub is_register: bool,
}

impl Destination {
    pub fn memory(index: u16) -> Destination {
        Destination {
            index,
            is_register: false,
        }
    }

    pub fn register(index: u16) -> Destination {
        Destination {
            index,
            is_register: true,
        }
    }

    pub fn as_address(&self, destination_type: TypeKind) -> Address {
        let kind = match (destination_type, self.is_register) {
            (TypeKind::Boolean, true) => AddressKind::BOOLEAN_REGISTER,
            (TypeKind::Boolean, false) => AddressKind::BOOLEAN_MEMORY,
            (TypeKind::Byte, true) => AddressKind::BYTE_REGISTER,
            (TypeKind::Byte, false) => AddressKind::BYTE_MEMORY,
            (TypeKind::Character, true) => AddressKind::CHARACTER_REGISTER,
            (TypeKind::Character, false) => AddressKind::CHARACTER_MEMORY,
            (TypeKind::Float, true) => AddressKind::FLOAT_REGISTER,
            (TypeKind::Float, false) => AddressKind::FLOAT_MEMORY,
            (TypeKind::Integer, true) => AddressKind::INTEGER_REGISTER,
            (TypeKind::Integer, false) => AddressKind::INTEGER_MEMORY,
            (TypeKind::String, true) => AddressKind::STRING_REGISTER,
            (TypeKind::String, false) => AddressKind::STRING_MEMORY,
            (TypeKind::List, true) => AddressKind::LIST_REGISTER,
            (TypeKind::List, false) => AddressKind::LIST_MEMORY,
            (TypeKind::Function, true) => AddressKind::FUNCTION_REGISTER,
            (TypeKind::Function, false) => AddressKind::FUNCTION_MEMORY,
            (TypeKind::None, _) => AddressKind::NONE,
            _ => todo!(),
        };

        Address {
            index: self.index,
            kind,
        }
    }

    pub fn display(&self, f: &mut Formatter, destination_type: TypeKind) -> fmt::Result {
        if self.is_register {
            write!(f, "R_")?;
        } else {
            write!(f, "M_")?;
        }

        match destination_type {
            TypeKind::Boolean => write!(f, "BOOL_")?,
            TypeKind::Byte => write!(f, "BYTE_")?,
            TypeKind::Character => write!(f, "CHAR_")?,
            TypeKind::Float => write!(f, "FLOAT_")?,
            TypeKind::Integer => write!(f, "INT_")?,
            TypeKind::String => write!(f, "STR_")?,
            TypeKind::List => write!(f, "LIST_")?,
            TypeKind::Function => write!(f, "FN_")?,
            invalid => invalid.write_invalid(f)?,
        }

        write!(f, "{}", self.index)
    }
}
