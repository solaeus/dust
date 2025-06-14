use std::{array, sync::Arc};

use hashbrown::HashSet;

use crate::{
    AbstractList, Address, Chunk, ConcreteList, DEFAULT_REGISTER_COUNT, DustString, Type,
    instruction::OperandType,
};

#[derive(Debug)]
pub struct Memory {
    pub booleans: Vec<bool>,
    pub bytes: Vec<u8>,
    pub characters: Vec<char>,
    pub floats: Vec<f64>,
    pub integers: Vec<i64>,
    pub strings: Vec<DustString>,
    pub lists: Vec<AbstractList>,
    pub functions: Vec<Arc<Chunk>>,

    pub closed: HashSet<Address>,

    pub registers: RegisterTable<DEFAULT_REGISTER_COUNT>,
}

impl Memory {
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

    pub fn make_list_concrete(&self, abstract_list: &AbstractList) -> ConcreteList {
        let item_type = abstract_list.item_type;

        match item_type {
            OperandType::BOOLEAN => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.booleans.get(*index as usize).copied())
                    .collect::<Vec<bool>>();

                ConcreteList::Boolean(list)
            }
            OperandType::BYTE => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.bytes.get(*index as usize).copied())
                    .collect::<Vec<u8>>();

                ConcreteList::Byte(list)
            }
            OperandType::CHARACTER => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.characters.get(*index as usize).copied())
                    .collect::<Vec<char>>();

                ConcreteList::Character(list)
            }
            OperandType::FLOAT => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.floats.get(*index as usize).copied())
                    .collect::<Vec<f64>>();

                ConcreteList::Float(list)
            }
            OperandType::INTEGER => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.integers.get(*index as usize).copied())
                    .collect::<Vec<i64>>();

                ConcreteList::Integer(list)
            }
            OperandType::STRING => {
                let list = abstract_list
                    .indices
                    .iter()
                    .filter_map(|index| self.strings.get(*index as usize).cloned())
                    .collect::<Vec<DustString>>();

                ConcreteList::String(list)
            }
            OperandType::LIST => {
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
            OperandType::NONE => ConcreteList::List {
                list_item_type: Type::None,
                list_items: Vec::with_capacity(0),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct RegisterTable<const LENGTH: usize> {
    pub booleans: [bool; LENGTH],
    pub bytes: [u8; LENGTH],
    pub characters: [char; LENGTH],
    pub floats: [f64; LENGTH],
    pub integers: [i64; LENGTH],
    pub strings: [DustString; LENGTH],
    pub lists: [AbstractList; LENGTH],
    pub functions: [Arc<Chunk>; LENGTH],
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
