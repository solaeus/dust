use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::instruction::ByteType;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum List {
    Boolean(Vec<bool>),
    Byte(Vec<u8>),
    Character(Vec<char>),
    Float(Vec<f64>),
    Integer(Vec<i64>),
    String(Vec<String>),
    Nested(Vec<List>),
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
        List::Nested(lists.into())
    }

    pub fn function<T: Into<Vec<usize>>>(prototype_indexes: T) -> Self {
        List::Function(prototype_indexes.into())
    }

    pub fn operand_type(&self) -> ByteType {
        match self {
            List::Boolean(_) => ByteType::LIST_BOOLEAN,
            List::Byte(_) => ByteType::LIST_BYTE,
            List::Character(_) => ByteType::LIST_CHARACTER,
            List::Float(_) => ByteType::LIST_FLOAT,
            List::Integer(_) => ByteType::LIST_INTEGER,
            List::String(_) => ByteType::LIST_STRING,
            List::Nested(_) => ByteType::LIST_LIST,
            List::Function(_) => ByteType::LIST_FUNCTION,
        }
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;

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

                    write!(f, "\"{string}\"")?;
                }
            }
            List::Nested(lists) => {
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

        write!(f, "]")
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
                    if left != right {
                        return false;
                    }
                }

                true
            }
            (List::Integer(left), List::Integer(right)) => left == right,
            (List::String(left), List::String(right)) => left == right,
            (List::Nested(left), List::Nested(right)) => left == right,
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
            (List::Nested(left), List::Nested(right)) => left.cmp(right),
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
            List::Nested(value) => value.hash(state),
            List::Function(value) => value.hash(state),
        }
    }
}
