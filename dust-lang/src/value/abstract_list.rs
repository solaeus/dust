use std::{
    fmt::{self, Formatter},
    sync::Arc,
};

use crate::{Address, instruction::TypeCode, risky_vm::Thread};

use super::ConcreteValue;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AbstractList {
    pub item_type: TypeCode,
    pub item_pointers: Vec<Address>,
}

impl AbstractList {
    pub fn display(&self, f: &mut Formatter, thread: &Thread) -> fmt::Result {
        write!(f, "[")?;

        for (i, pointer) in self.item_pointers.iter().copied().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            match pointer.as_type_code() {
                TypeCode::BOOLEAN => {
                    let boolean = thread.current_memory.booleans[pointer.index as usize].as_value();

                    write!(f, "{boolean}")?;
                }
                TypeCode::BYTE => {
                    let byte = thread.current_memory.bytes[pointer.index as usize].as_value();

                    write!(f, "{byte}")?;
                }
                TypeCode::CHARACTER => {
                    let character =
                        thread.current_memory.characters[pointer.index as usize].as_value();

                    write!(f, "{character}")?;
                }
                TypeCode::FLOAT => {
                    let float = thread.current_memory.floats[pointer.index as usize].as_value();

                    write!(f, "{float}")?;
                }
                TypeCode::INTEGER => {
                    let integer = thread.current_memory.integers[pointer.index as usize].as_value();

                    write!(f, "{integer}")?;
                }
                TypeCode::STRING => {
                    let string = thread.current_memory.strings[pointer.index as usize].as_value();

                    write!(f, "{string}")?;
                }
                TypeCode::LIST => {
                    let list = thread.current_memory.lists[pointer.index as usize].as_value();

                    list.display(f, thread)?;
                }
                TypeCode::FUNCTION => {
                    let function =
                        thread.current_memory.functions[pointer.index as usize].as_value();

                    function.display(f, thread)?;
                }
                _ => write!(f, "INVALID")?,
            }
        }

        write!(f, "]")
    }

    pub fn to_concrete(&self, thread: &Thread) -> ConcreteValue {
        let mut concrete_list = Vec::with_capacity(self.item_pointers.len());

        match self.item_type {
            TypeCode::BOOLEAN => {
                for pointer in &self.item_pointers {
                    let boolean = *thread
                        .current_memory
                        .booleans
                        .get(pointer.index as usize)
                        .unwrap()
                        .as_value();

                    concrete_list.push(ConcreteValue::Boolean(boolean));
                }
            }
            TypeCode::BYTE => {
                for pointer in &self.item_pointers {
                    let byte = *thread
                        .current_memory
                        .bytes
                        .get(pointer.index as usize)
                        .unwrap()
                        .as_value();

                    concrete_list.push(ConcreteValue::Byte(byte));
                }
            }
            TypeCode::CHARACTER => {
                for pointer in &self.item_pointers {
                    let character = if pointer.is_constant() {
                        *thread
                            .current_call
                            .chunk
                            .character_constants
                            .get(pointer.index as usize)
                            .unwrap()
                    } else if pointer.is_register() {
                        *thread
                            .current_memory
                            .register_table
                            .characters
                            .get(pointer.index)
                    } else {
                        *thread
                            .current_memory
                            .characters
                            .get(pointer.index as usize)
                            .unwrap()
                            .as_value()
                    };

                    concrete_list.push(ConcreteValue::Character(character));
                }
            }
            TypeCode::FLOAT => {
                for pointer in &self.item_pointers {
                    let float = if pointer.is_constant() {
                        *thread
                            .current_call
                            .chunk
                            .float_constants
                            .get(pointer.index as usize)
                            .unwrap()
                    } else if pointer.is_register() {
                        *thread
                            .current_memory
                            .register_table
                            .floats
                            .get(pointer.index)
                    } else {
                        *thread
                            .current_memory
                            .floats
                            .get(pointer.index as usize)
                            .unwrap()
                            .as_value()
                    };

                    concrete_list.push(ConcreteValue::Float(float));
                }
            }
            TypeCode::INTEGER => {
                for pointer in &self.item_pointers {
                    let integer = if pointer.is_constant() {
                        *thread
                            .current_call
                            .chunk
                            .integer_constants
                            .get(pointer.index as usize)
                            .unwrap()
                    } else if pointer.is_register() {
                        *thread
                            .current_memory
                            .register_table
                            .integers
                            .get(pointer.index)
                    } else {
                        *thread
                            .current_memory
                            .integers
                            .get(pointer.index as usize)
                            .unwrap()
                            .as_value()
                    };

                    concrete_list.push(ConcreteValue::Integer(integer));
                }
            }
            TypeCode::STRING => {
                for pointer in &self.item_pointers {
                    let string = if pointer.is_constant() {
                        thread
                            .current_call
                            .chunk
                            .string_constants
                            .get(pointer.index as usize)
                            .unwrap()
                            .clone()
                    } else if pointer.is_register() {
                        thread
                            .current_memory
                            .register_table
                            .strings
                            .get(pointer.index)
                            .clone()
                    } else {
                        thread
                            .current_memory
                            .strings
                            .get(pointer.index as usize)
                            .unwrap()
                            .as_value()
                            .clone()
                    };

                    concrete_list.push(ConcreteValue::String(string));
                }
            }
            TypeCode::LIST => {
                for pointer in &self.item_pointers {
                    let list = thread
                        .current_memory
                        .lists
                        .get(pointer.index as usize)
                        .unwrap()
                        .as_value()
                        .to_concrete(thread);

                    concrete_list.push(list);
                }
            }
            TypeCode::FUNCTION => {
                for pointer in &self.item_pointers {
                    let prototype_index = thread
                        .current_memory
                        .functions
                        .get(pointer.index as usize)
                        .unwrap()
                        .as_value()
                        .prototype_index as usize;
                    let chunk = thread
                        .current_call
                        .chunk
                        .prototypes
                        .get(prototype_index)
                        .unwrap();

                    concrete_list.push(ConcreteValue::Function(Arc::clone(chunk)));
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
