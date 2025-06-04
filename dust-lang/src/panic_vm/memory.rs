use std::{array, sync::Arc};

use hashbrown::HashSet;

use crate::{AbstractList, Address, Chunk, ConcreteList, DustString, Type, r#type::TypeKind};

#[derive(Debug)]
pub struct Memory<const REGISTER_COUNT: usize> {
    booleans: Vec<bool>,
    bytes: Vec<u8>,
    characters: Vec<char>,
    floats: Vec<f64>,
    integers: Vec<i64>,
    strings: Vec<DustString>,
    lists: Vec<AbstractList>,
    functions: Vec<Arc<Chunk>>,

    pub closed: HashSet<Address>,

    registers: RegisterTable<REGISTER_COUNT>,
}

impl<const REGISTER_COUNT: usize> Memory<REGISTER_COUNT> {
    #[expect(clippy::rc_clone_in_vec_init)]
    pub fn new(chunk: &Chunk) -> Self {
        Memory {
            booleans: vec![false; chunk.boolean_memory_length as usize],
            bytes: vec![0; chunk.byte_memory_length as usize],
            characters: vec![char::default(); chunk.character_memory_length as usize],
            floats: vec![0.0; chunk.float_memory_length as usize],
            integers: vec![0; chunk.integer_memory_length as usize],
            strings: vec![DustString::new(); chunk.string_memory_length as usize],
            lists: vec![AbstractList::default(); chunk.list_memory_length as usize],
            functions: vec![Arc::new(Chunk::default()); chunk.function_memory_length as usize],
            closed: HashSet::new(),
            registers: RegisterTable::new(),
        }
    }

    pub fn get_boolean(&self, is_register: bool, index: u16) -> bool {
        if is_register {
            self.registers.booleans[index as usize]
        } else {
            self.booleans[index as usize]
        }
    }

    pub fn set_boolean(&mut self, is_register: bool, index: u16, value: bool) {
        if is_register {
            self.registers.booleans[index as usize] = value;
        } else {
            self.booleans[index as usize] = value;
        }
    }

    pub fn get_byte(&self, is_register: bool, index: u16) -> u8 {
        if is_register {
            self.registers.bytes[index as usize]
        } else {
            self.bytes[index as usize]
        }
    }

    pub fn set_byte(&mut self, is_register: bool, index: u16, value: u8) {
        if is_register {
            self.registers.bytes[index as usize] = value;
        } else {
            self.bytes[index as usize] = value;
        }
    }

    pub fn get_character(&self, is_register: bool, index: u16) -> char {
        if is_register {
            self.registers.characters[index as usize]
        } else {
            self.characters[index as usize]
        }
    }

    pub fn set_character(&mut self, is_register: bool, index: u16, value: char) {
        if is_register {
            self.registers.characters[index as usize] = value;
        } else {
            self.characters[index as usize] = value;
        }
    }

    pub fn get_float(&self, is_register: bool, index: u16) -> f64 {
        if is_register {
            self.registers.floats[index as usize]
        } else {
            self.floats[index as usize]
        }
    }

    pub fn set_float(&mut self, is_register: bool, index: u16, value: f64) {
        if is_register {
            self.registers.floats[index as usize] = value;
        } else {
            self.floats[index as usize] = value;
        }
    }

    pub fn get_integer(&self, is_register: bool, index: u16) -> i64 {
        if is_register {
            self.registers.integers[index as usize]
        } else {
            self.integers[index as usize]
        }
    }

    pub fn set_integer(&mut self, is_register: bool, index: u16, value: i64) {
        if is_register {
            self.registers.integers[index as usize] = value;
        } else {
            self.integers[index as usize] = value;
        }
    }

    pub fn get_string(&self, is_register: bool, index: u16) -> &DustString {
        if is_register {
            &self.registers.strings[index as usize]
        } else {
            &self.strings[index as usize]
        }
    }

    pub fn set_string(&mut self, is_register: bool, index: u16, value: DustString) {
        if is_register {
            self.registers.strings[index as usize] = value;
        } else {
            self.strings[index as usize] = value;
        }
    }

    pub fn get_list(&self, is_register: bool, index: u16) -> &AbstractList {
        if is_register {
            &self.registers.lists[index as usize]
        } else {
            &self.lists[index as usize]
        }
    }

    pub fn set_list(&mut self, is_register: bool, index: u16, value: AbstractList) {
        if is_register {
            self.registers.lists[index as usize] = value;
        } else {
            self.lists[index as usize] = value;
        }
    }

    pub fn get_function(&self, is_register: bool, index: u16) -> &Arc<Chunk> {
        if is_register {
            &self.registers.functions[index as usize]
        } else {
            &self.functions[index as usize]
        }
    }

    pub fn set_function(&mut self, is_register: bool, index: u16, value: Arc<Chunk>) {
        if is_register {
            self.registers.functions[index as usize] = value;
        } else {
            self.functions[index as usize] = value;
        }
    }

    pub fn make_list_concrete(&self, abstract_list: &AbstractList) -> ConcreteList {
        let item_type = abstract_list.pointer_kind.r#type();

        match item_type {
            TypeKind::Boolean => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.booleans.get(*index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Boolean(list)
            }
            TypeKind::Byte => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.bytes.get(*index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Byte(list)
            }
            TypeKind::Character => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.characters.get(*index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Character(list)
            }
            TypeKind::Float => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.floats.get(*index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Float(list)
            }
            TypeKind::Integer => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.integers.get(*index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Integer(list)
            }
            TypeKind::String => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.strings.get(*index as usize).cloned())
                    .collect::<Vec<_>>();

                ConcreteList::String(list)
            }
            TypeKind::List => {
                let list = abstract_list
                    .indices
                    .iter()
                    .map(|index| {
                        let abstract_list = &self.lists[*index as usize];

                        self.make_list_concrete(abstract_list)
                    })
                    .collect::<Vec<_>>();

                let list_item_type = list.first().map(|list| list.r#type()).unwrap_or(Type::None);

                ConcreteList::List {
                    list_item_type,
                    list_items: list,
                }
            }
            TypeKind::None => ConcreteList::List {
                list_item_type: Type::None,
                list_items: Vec::with_capacity(0),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct RegisterTable<const LENGTH: usize> {
    booleans: [bool; LENGTH],
    bytes: [u8; LENGTH],
    characters: [char; LENGTH],
    floats: [f64; LENGTH],
    integers: [i64; LENGTH],
    strings: [DustString; LENGTH],
    lists: [AbstractList; LENGTH],
    functions: [Arc<Chunk>; LENGTH],
}

impl<const LENGTH: usize> RegisterTable<LENGTH> {
    pub fn new() -> Self {
        RegisterTable {
            booleans: [false; LENGTH],
            bytes: [0; LENGTH],
            characters: [char::default(); LENGTH],
            floats: [0.0; LENGTH],
            integers: [0; LENGTH],
            strings: array::from_fn(|_| DustString::new()),
            lists: array::from_fn(|_| AbstractList::default()),
            functions: array::from_fn(|_| Arc::new(Chunk::default())),
        }
    }
}

impl<const LENGTH: usize> Default for RegisterTable<LENGTH> {
    fn default() -> Self {
        Self::new()
    }
}
