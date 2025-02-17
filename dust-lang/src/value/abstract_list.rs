use std::fmt::{self, Display, Formatter};

use crate::{
    instruction::TypeCode,
    vm::{Pointer, Thread},
};

use super::{ConcreteValue, DustString};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AbstractList {
    pub item_type: TypeCode,
    pub item_pointers: Vec<Pointer>,
}

impl AbstractList {
    pub fn display(&self, thread: &Thread) -> DustString {
        let current_frame = thread.current_frame();
        let mut display = DustString::new();

        display.push('[');

        for (i, pointer) in self.item_pointers.iter().copied().enumerate() {
            if i > 0 {
                display.push_str(", ");
            }

            let item_display = match self.item_type {
                TypeCode::BOOLEAN => current_frame.get_boolean_from_pointer(pointer).to_string(),
                TypeCode::BYTE => current_frame.get_byte_from_pointer(pointer).to_string(),
                TypeCode::CHARACTER => current_frame
                    .get_character_from_pointer(pointer)
                    .to_string(),
                TypeCode::FLOAT => current_frame.get_float_from_pointer(pointer).to_string(),
                TypeCode::INTEGER => current_frame.get_integer_from_pointer(pointer).to_string(),
                TypeCode::STRING => current_frame.get_string_from_pointer(pointer).to_string(),
                _ => todo!(),
            };

            display.push_str(&item_display);
        }

        display.push(']');

        display
    }

    pub fn to_concrete(&self, thread: &Thread) -> ConcreteValue {
        let mut concrete_list = Vec::with_capacity(self.item_pointers.len());

        match self.item_type {
            TypeCode::BOOLEAN => {
                for pointer in &self.item_pointers {
                    let boolean = thread.current_frame().get_boolean_from_pointer(*pointer);

                    concrete_list.push(ConcreteValue::Boolean(boolean));
                }
            }
            TypeCode::BYTE => {
                for pointer in &self.item_pointers {
                    let byte = thread.current_frame().get_byte_from_pointer(*pointer);

                    concrete_list.push(ConcreteValue::Byte(byte));
                }
            }
            TypeCode::CHARACTER => {
                for pointer in &self.item_pointers {
                    let character = thread.current_frame().get_character_from_pointer(*pointer);

                    concrete_list.push(ConcreteValue::Character(character));
                }
            }
            TypeCode::FLOAT => {
                for pointer in &self.item_pointers {
                    let float = thread.current_frame().get_float_from_pointer(*pointer);

                    concrete_list.push(ConcreteValue::Float(float));
                }
            }
            TypeCode::INTEGER => {
                for pointer in &self.item_pointers {
                    let integer = thread.current_frame().get_integer_from_pointer(*pointer);

                    concrete_list.push(ConcreteValue::Integer(integer));
                }
            }
            TypeCode::STRING => {
                for pointer in &self.item_pointers {
                    let string = thread
                        .current_frame()
                        .get_string_from_pointer(*pointer)
                        .clone();

                    concrete_list.push(ConcreteValue::String(string));
                }
            }
            TypeCode::LIST => {
                for pointer in &self.item_pointers {
                    let list = thread
                        .current_frame()
                        .get_list_from_pointer(pointer)
                        .to_concrete(thread);

                    concrete_list.push(list);
                }
            }
            _ => todo!(),
        }

        ConcreteValue::List(concrete_list)
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
            write!(f, "{:?}", pointer)?;
        }

        write!(f, "]")
    }
}
