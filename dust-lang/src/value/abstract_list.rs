use std::fmt::{self, Display, Formatter};

use crate::{vm::Record, Pointer, Type};

use super::DustString;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AbstractList {
    pub item_type: Type,
    pub item_pointers: Vec<Pointer>,
}

impl AbstractList {
    pub fn display(&self, record: &Record) -> DustString {
        let mut display = DustString::new();

        display.push('[');

        for (i, item) in self.item_pointers.iter().enumerate() {
            if i > 0 {
                display.push_str(", ");
            }

            let item_display = record.follow_pointer(*item).display(record);

            display.push_str(&item_display);
        }

        display.push(']');

        display
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
