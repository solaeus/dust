use std::fmt::{self, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Address, risky_vm::Thread, r#type::TypeKind, value::concrete_value::ConcreteList};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AbstractList {
    pub item_type: TypeKind,
    pub item_pointers: Vec<Address>,
}

impl AbstractList {
    pub fn display(&self, f: &mut Formatter, thread: &Thread) -> fmt::Result {
        write!(f, "[")?;

        for (i, pointer) in self.item_pointers.iter().copied().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            match pointer.r#type() {
                TypeKind::Boolean => {
                    let boolean = thread.current_memory.booleans[pointer.index as usize].as_value();

                    write!(f, "{boolean}")?;
                }
                TypeKind::Byte => {
                    let byte = thread.current_memory.bytes[pointer.index as usize].as_value();

                    write!(f, "{byte}")?;
                }
                TypeKind::Character => {
                    let character =
                        thread.current_memory.characters[pointer.index as usize].as_value();

                    write!(f, "{character}")?;
                }
                TypeKind::Float => {
                    let float = thread.current_memory.floats[pointer.index as usize].as_value();

                    write!(f, "{float}")?;
                }
                TypeKind::Integer => {
                    let integer = thread.current_memory.integers[pointer.index as usize].as_value();

                    write!(f, "{integer}")?;
                }
                TypeKind::String => {
                    let string = thread.current_memory.strings[pointer.index as usize].as_value();

                    write!(f, "{string}")?;
                }
                TypeKind::List => {
                    let list = thread.current_memory.lists[pointer.index as usize].as_value();

                    list.display(f, thread)?;
                }
                TypeKind::Function => {
                    let function =
                        thread.current_memory.functions[pointer.index as usize].as_value();

                    function.display(f, thread)?;
                }
                _ => write!(f, "INVALID")?,
            }
        }

        write!(f, "]")
    }

    pub fn to_concrete(&self, thread: &Thread) -> ConcreteList {
        match self.item_type {
            TypeKind::Boolean => {
                let mut boolean_list = Vec::with_capacity(self.item_pointers.len());

                for pointer in &self.item_pointers {
                    let boolean = thread.resolve_boolean(pointer);

                    boolean_list.push(boolean);
                }

                ConcreteList::Boolean(boolean_list)
            }
            TypeKind::Byte => {
                let mut byte_list = Vec::with_capacity(self.item_pointers.len());

                for pointer in &self.item_pointers {
                    let byte = if pointer.is_register() {
                        *thread
                            .current_memory
                            .register_table
                            .bytes
                            .get(pointer.index)
                    } else {
                        *thread
                            .current_memory
                            .bytes
                            .get(pointer.index as usize)
                            .unwrap()
                            .as_value()
                    };

                    byte_list.push(byte);
                }

                ConcreteList::Byte(byte_list)
            }
            TypeKind::Character => {
                todo!()
            }
            TypeKind::Float => {
                todo!()
            }
            TypeKind::Integer => {
                todo!()
            }
            TypeKind::String => {
                todo!()
            }
            TypeKind::List => {
                todo!()
            }
            TypeKind::Function => {
                todo!()
            }
            _ => todo!(),
        }
    }
}

impl Default for AbstractList {
    fn default() -> Self {
        Self {
            item_type: TypeKind::None,
            item_pointers: Vec::new(),
        }
    }
}
