use std::{array, sync::Arc};

use hashbrown::HashSet;

use crate::{
    AbstractList, Address, Chunk, ConcreteList, DustString, instruction::AddressKind,
    r#type::TypeKind,
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

    pub registers: RegisterTable,
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
        let item_type = abstract_list
            .item_pointers
            .first()
            .map(|pointer| pointer.r#type())
            .expect("Cannot make an empty abstract list into a concrete list");

        match item_type {
            TypeKind::Boolean => {
                let list = abstract_list
                    .item_pointers
                    .iter()
                    .filter_map(|pointer| self.booleans.get(pointer.index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Boolean(list)
            }
            TypeKind::Byte => {
                let list = abstract_list
                    .item_pointers
                    .iter()
                    .filter_map(|pointer| self.bytes.get(pointer.index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Byte(list)
            }
            TypeKind::Character => {
                let list = abstract_list
                    .item_pointers
                    .iter()
                    .filter_map(|pointer| self.characters.get(pointer.index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Character(list)
            }
            TypeKind::Float => {
                let list = abstract_list
                    .item_pointers
                    .iter()
                    .filter_map(|pointer| self.floats.get(pointer.index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Float(list)
            }
            TypeKind::Integer => {
                let list = abstract_list
                    .item_pointers
                    .iter()
                    .filter_map(|pointer| self.integers.get(pointer.index as usize).copied())
                    .collect::<Vec<_>>();

                ConcreteList::Integer(list)
            }
            TypeKind::String => {
                let list = abstract_list
                    .item_pointers
                    .iter()
                    .filter_map(|pointer| self.strings.get(pointer.index as usize).cloned())
                    .collect::<Vec<_>>();

                ConcreteList::String(list)
            }
            TypeKind::List => {
                let lists = abstract_list
                    .item_pointers
                    .iter()
                    .map(|pointer| {
                        let abstract_list = &self.lists[pointer.index as usize];

                        self.make_list_concrete(abstract_list)
                    })
                    .collect::<Vec<_>>();

                ConcreteList::List {
                    list_item_type: lists.first().unwrap().r#type(),
                    list_items: lists,
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct RegisterTable<const LENGTH: usize = 3> {
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
            lists: array::from_fn(|_| AbstractList {
                item_pointers: Vec::with_capacity(0),
            }),
            functions: array::from_fn(|_| Arc::new(Chunk::default())),
        }
    }
}

impl<const LENGTH: usize> Default for RegisterTable<LENGTH> {
    fn default() -> Self {
        Self::new()
    }
}
