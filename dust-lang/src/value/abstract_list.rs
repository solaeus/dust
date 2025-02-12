use std::fmt::{self, Display, Formatter};

use crate::{
    instruction::TypeCode,
    vm::{Pointer, Thread},
};

use super::DustString;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AbstractList {
    pub item_type: TypeCode,
    pub item_pointers: Vec<Pointer>,
}

impl AbstractList {
    pub fn display(&self, thread: &Thread) -> DustString {
        let mut display = DustString::new();

        display.push('[');

        for (i, pointer) in self.item_pointers.iter().enumerate() {
            if i > 0 {
                display.push_str(", ");
            }

            let item_display = match self.item_type {
                TypeCode::BOOLEAN => thread.get_pointer_to_boolean(pointer).to_string(),
                TypeCode::BYTE => thread.get_pointer_to_byte(pointer).to_string(),
                TypeCode::CHARACTER => thread.get_pointer_to_character(pointer).to_string(),
                TypeCode::FLOAT => thread.get_pointer_to_float(pointer).to_string(),
                TypeCode::INTEGER => thread.get_pointer_to_integer(pointer).to_string(),
                TypeCode::STRING => thread.get_pointer_to_string(pointer).to_string(),
                _ => todo!(),
            };

            display.push_str(&item_display);
        }

        display.push(']');

        display
    }
}

impl Default for AbstractList {
    fn default() -> Self {
        Self {
            item_type: TypeCode::NONE,
            item_pointers: Vec::new(),
        }
    }
}

impl Display for AbstractList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;

        for pointer in &self.item_pointers {
            write!(f, "{}", pointer)?;
        }

        write!(f, "]")
    }
}
