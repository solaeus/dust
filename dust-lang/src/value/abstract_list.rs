use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use crate::{
    instruction::TypeCode,
    risky_vm::{Pointer, Thread},
};

use super::ConcreteValue;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AbstractList {
    pub item_type: TypeCode,
    pub item_pointers: Vec<Pointer>,
}

impl AbstractList {
    pub fn display(&self, f: &mut Formatter, thread: &Thread) -> fmt::Result {
        write!(f, "[")?;

        for (i, pointer) in self.item_pointers.iter().copied().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            match (pointer, self.item_type) {
                (Pointer::Register(register_index), TypeCode::BOOLEAN) => thread
                    .current_registers()
                    .booleans
                    .get(register_index as usize)
                    .as_value()
                    .fmt(f)?,
                (Pointer::Register(register_index), TypeCode::BYTE) => thread
                    .current_registers()
                    .bytes
                    .get(register_index as usize)
                    .as_value()
                    .fmt(f)?,
                (Pointer::Constant(constant_index), TypeCode::CHARACTER) => thread
                    .current_frame()
                    .chunk
                    .character_constants
                    .get(constant_index as usize)
                    .unwrap()
                    .fmt(f)?,
                (Pointer::Register(register_index), TypeCode::CHARACTER) => thread
                    .current_registers()
                    .characters
                    .get(register_index as usize)
                    .as_value()
                    .fmt(f)?,
                (Pointer::Constant(constant_index), TypeCode::FLOAT) => thread
                    .current_frame()
                    .chunk
                    .float_constants
                    .get(constant_index as usize)
                    .unwrap()
                    .fmt(f)?,
                (Pointer::Register(register_index), TypeCode::FLOAT) => thread
                    .current_registers()
                    .floats
                    .get(register_index as usize)
                    .as_value()
                    .fmt(f)?,
                (Pointer::Constant(constant_index), TypeCode::INTEGER) => thread
                    .current_frame()
                    .chunk
                    .integer_constants
                    .get(constant_index as usize)
                    .unwrap()
                    .fmt(f)?,
                (Pointer::Register(register_index), TypeCode::INTEGER) => thread
                    .current_registers()
                    .integers
                    .get(register_index as usize)
                    .as_value()
                    .fmt(f)?,
                (Pointer::Constant(constant_index), TypeCode::STRING) => thread
                    .current_frame()
                    .chunk
                    .string_constants
                    .get(constant_index as usize)
                    .unwrap()
                    .fmt(f)?,
                (Pointer::Register(register_index), TypeCode::STRING) => thread
                    .current_registers()
                    .strings
                    .get(register_index as usize)
                    .as_value()
                    .fmt(f)?,
                (Pointer::Register(register_index), TypeCode::LIST) => thread
                    .current_registers()
                    .lists
                    .get(register_index as usize)
                    .as_value()
                    .display(f, thread)?,
                _ => todo!(),
            };
        }

        write!(f, "]")
    }

    pub fn to_concrete(&self, thread: &Thread) -> ConcreteValue {
        let mut concrete_list = Vec::with_capacity(self.item_pointers.len());

        match self.item_type {
            TypeCode::BOOLEAN => {
                for pointer in &self.item_pointers {
                    let boolean = *thread
                        .current_registers()
                        .booleans
                        .get(pointer.index() as usize)
                        .as_value();

                    concrete_list.push(ConcreteValue::Boolean(boolean));
                }
            }
            TypeCode::BYTE => {
                for pointer in &self.item_pointers {
                    let byte = *thread
                        .current_registers()
                        .bytes
                        .get(pointer.index() as usize)
                        .as_value();

                    concrete_list.push(ConcreteValue::Byte(byte));
                }
            }
            TypeCode::CHARACTER => {
                for pointer in &self.item_pointers {
                    let character = match pointer {
                        Pointer::Register(register_index) => *thread
                            .current_registers()
                            .characters
                            .get(*register_index as usize)
                            .as_value(),
                        Pointer::Constant(constant_index) => *thread
                            .current_frame()
                            .chunk
                            .character_constants
                            .get(*constant_index as usize)
                            .unwrap(),
                    };

                    concrete_list.push(ConcreteValue::Character(character));
                }
            }
            TypeCode::FLOAT => {
                for pointer in &self.item_pointers {
                    let float = match pointer {
                        Pointer::Register(register_index) => *thread
                            .current_registers()
                            .floats
                            .get(*register_index as usize)
                            .as_value(),
                        Pointer::Constant(constant_index) => *thread
                            .current_frame()
                            .chunk
                            .float_constants
                            .get(*constant_index as usize)
                            .unwrap(),
                    };

                    concrete_list.push(ConcreteValue::Float(float));
                }
            }
            TypeCode::INTEGER => {
                for pointer in &self.item_pointers {
                    let integer = match pointer {
                        Pointer::Register(register_index) => *thread
                            .current_registers()
                            .integers
                            .get(*register_index as usize)
                            .as_value(),
                        Pointer::Constant(constant_index) => *thread
                            .current_frame()
                            .chunk
                            .integer_constants
                            .get(*constant_index as usize)
                            .unwrap(),
                    };

                    concrete_list.push(ConcreteValue::Integer(integer));
                }
            }
            TypeCode::STRING => {
                for pointer in &self.item_pointers {
                    let string = match pointer {
                        Pointer::Register(register_index) => {
                            let string = thread
                                .current_registers()
                                .strings
                                .get(*register_index as usize)
                                .as_value();

                            string.clone()
                        }
                        Pointer::Constant(constant_index) => {
                            let string = thread
                                .current_frame()
                                .chunk
                                .string_constants
                                .get(*constant_index as usize)
                                .unwrap();

                            string.clone()
                        }
                    };

                    concrete_list.push(ConcreteValue::String(string));
                }
            }
            TypeCode::LIST => {
                for pointer in &self.item_pointers {
                    let list = thread
                        .current_registers()
                        .lists
                        .get(pointer.index() as usize)
                        .as_value()
                        .to_concrete(thread);

                    concrete_list.push(list);
                }
            }
            TypeCode::FUNCTION => {
                for pointer in &self.item_pointers {
                    let prototype_index = thread
                        .current_registers()
                        .functions
                        .get(pointer.index() as usize)
                        .as_value()
                        .prototype_index as usize;
                    let chunk = thread
                        .current_frame()
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
