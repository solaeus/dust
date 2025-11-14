use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::instruction::OperandType;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum List {
    Boolean(Vec<bool>),
    Byte(Vec<u8>),
    Character(Vec<char>),
    Float(Vec<f64>),
    Integer(Vec<i64>),
    String(Vec<String>),
    List(Vec<List>),
    Function(Vec<usize>),
}

impl List {
    pub fn boolean<T: Into<Vec<bool>>>(booleans: T) -> Self {
        List::Boolean(booleans.into())
    }

    pub fn byte<T: Into<Vec<u8>>>(bytes: T) -> Self {
        List::Byte(bytes.into())
    }

    pub fn character<T: Into<Vec<char>>>(characters: T) -> Self {
        List::Character(characters.into())
    }

    pub fn float<T: Into<Vec<f64>>>(floats: T) -> Self {
        List::Float(floats.into())
    }

    pub fn integer<T: Into<Vec<i64>>>(items: T) -> Self {
        List::Integer(items.into())
    }

    pub fn string<T: Into<Vec<String>>>(strings: T) -> Self {
        List::String(strings.into())
    }

    #[expect(clippy::self_named_constructors)]
    pub fn list<T: Into<Vec<List>>>(lists: T) -> Self {
        List::List(lists.into())
    }

    pub fn function<T: Into<Vec<usize>>>(prototype_indexes: T) -> Self {
        List::Function(prototype_indexes.into())
    }

    pub fn operand_type(&self) -> OperandType {
        match self {
            List::Boolean(_) => OperandType::LIST_BOOLEAN,
            List::Byte(_) => OperandType::LIST_BYTE,
            List::Character(_) => OperandType::LIST_CHARACTER,
            List::Float(_) => OperandType::LIST_FLOAT,
            List::Integer(_) => OperandType::LIST_INTEGER,
            List::String(_) => OperandType::LIST_STRING,
            List::List(_) => OperandType::LIST_LIST,
            List::Function(_) => OperandType::LIST_FUNCTION,
        }
    }

    pub fn heap_size(&self) -> usize {
        match self {
            List::Boolean(booleans) => {
                let vec_stack_size = std::mem::size_of::<Vec<bool>>();
                let vec_heap_size = booleans.capacity() * std::mem::size_of::<bool>();

                vec_stack_size + vec_heap_size
            }
            List::Byte(bytes) => {
                let vec_stack_size = std::mem::size_of::<Vec<u8>>();
                let vec_heap_size = bytes.capacity() * std::mem::size_of::<u8>();

                vec_stack_size + vec_heap_size
            }
            List::Character(characters) => {
                let vec_stack_size = std::mem::size_of::<Vec<char>>();
                let vec_heap_size = characters.capacity() * std::mem::size_of::<char>();

                vec_stack_size + vec_heap_size
            }
            List::Float(floats) => {
                let vec_stack_size = std::mem::size_of::<Vec<f64>>();
                let vec_heap_size = floats.capacity() * std::mem::size_of::<f64>();

                vec_stack_size + vec_heap_size
            }
            List::Integer(items) => {
                let vec_stack_size = std::mem::size_of::<Vec<i64>>();
                let vec_heap_size = items.capacity() * std::mem::size_of::<i64>();

                vec_stack_size + vec_heap_size
            }
            List::String(strings) => {
                let vec_stack_size = std::mem::size_of::<Vec<String>>();
                let vec_heap_size = strings.capacity() * std::mem::size_of::<String>();
                let strings_heap_size = strings
                    .iter()
                    .map(|string| string.capacity())
                    .sum::<usize>();

                vec_stack_size + vec_heap_size + strings_heap_size
            }
            List::List(lists) => {
                let vec_stack_size = std::mem::size_of::<Vec<List>>();
                let vec_heap_size = lists.capacity() * std::mem::size_of::<List>();
                let lists_heap_size = lists.iter().map(List::heap_size).sum::<usize>();

                vec_stack_size + vec_heap_size + lists_heap_size
            }
            List::Function(functions) => {
                let vec_stack_size = std::mem::size_of::<Vec<usize>>();
                let vec_heap_size = functions.capacity() * std::mem::size_of::<usize>();

                vec_stack_size + vec_heap_size
            }
        }
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "list(")?;

        match self {
            List::Boolean(booleans) => {
                for (index, boolean) in booleans.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{boolean}")?;
                }
            }
            List::Byte(bytes) => {
                for (index, byte) in bytes.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{byte}")?;
                }
            }
            List::Character(characters) => {
                for (index, character) in characters.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{character}")?;
                }
            }
            List::Float(floats) => {
                for (index, float) in floats.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{float}")?;
                }
            }
            List::Integer(items) => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{item}")?;
                }
            }
            List::String(strings) => {
                for (index, string) in strings.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{string}")?;
                }
            }
            List::List(lists) => {
                for (index, list) in lists.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{list}")?;
                }
            }
            List::Function(functions) => {
                for (index, prototype) in functions.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{prototype}")?;
                }
            }
        }

        write!(f, ")")
    }
}

impl Eq for List {}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (List::Boolean(left), List::Boolean(right)) => left == right,
            (List::Byte(left), List::Byte(right)) => left == right,
            (List::Character(left), List::Character(right)) => left == right,
            (List::Float(left), List::Float(right)) => {
                for (left, right) in left.iter().zip(right.iter()) {
                    if left.to_bits() != right.to_bits() {
                        return false;
                    }
                }

                true
            }
            (List::Integer(left), List::Integer(right)) => left == right,
            (List::String(left), List::String(right)) => left == right,
            (List::List(left), List::List(right)) => left == right,
            (List::Function(left), List::Function(right)) => left == right,
            _ => false,
        }
    }
}

impl PartialOrd for List {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for List {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (List::Boolean(left), List::Boolean(right)) => left.cmp(right),
            (List::Byte(left), List::Byte(right)) => left.cmp(right),
            (List::Character(left), List::Character(right)) => left.cmp(right),
            (List::Float(left), List::Float(right)) => {
                for (left, right) in left.iter().zip(right.iter()) {
                    let cmp = left.total_cmp(right);

                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }

                Ordering::Equal
            }
            (List::Integer(left), List::Integer(right)) => left.cmp(right),
            (List::String(left), List::String(right)) => left.cmp(right),
            (List::List(left), List::List(right)) => left.cmp(right),
            (List::Function(left), List::Function(right)) => left.cmp(right),
            _ => Ordering::Equal,
        }
    }
}

impl Hash for List {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            List::Boolean(value) => value.hash(state),
            List::Byte(value) => value.hash(state),
            List::Character(value) => value.hash(state),
            List::Float(value) => {
                for float in value {
                    float.to_bits().hash(state);
                }
            }
            List::Integer(value) => value.hash(state),
            List::String(value) => value.hash(state),
            List::List(value) => value.hash(state),
            List::Function(value) => value.hash(state),
        }
    }
}
