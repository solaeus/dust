use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::{DustString, Function, Type};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum List {
    Boolean(Vec<bool>),
    Byte(Vec<u8>),
    Character(Vec<char>),
    Float(Vec<f64>),
    Integer(Vec<i64>),
    String(Vec<DustString>),
    List(Vec<List>),
    Function(Vec<Function>),
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

    pub fn string<T: Into<Vec<DustString>>>(strings: T) -> Self {
        List::String(strings.into())
    }

    #[expect(clippy::self_named_constructors)]
    pub fn list<T: Into<Vec<List>>>(lists: T) -> Self {
        List::List(lists.into())
    }

    pub fn function<T: Into<Vec<Function>>>(functions: T) -> Self {
        List::Function(functions.into())
    }

    pub fn item_type(&self) -> Type {
        match self {
            List::Boolean(_) => Type::Boolean,
            List::Byte(_) => Type::Byte,
            List::Character(_) => Type::Character,
            List::Float(_) => Type::Float,
            List::Integer(_) => Type::Integer,
            List::String(_) => Type::String,
            List::List(lists) => lists
                .first()
                .map(|list| list.r#type())
                .unwrap_or(Type::None),
            List::Function(functions) => functions
                .first()
                .map(|function| Type::Function(Box::new(function.r#type.clone())))
                .unwrap_or(Type::None),
        }
    }

    pub fn r#type(&self) -> Type {
        Type::List(Box::new(self.item_type()))
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
                for (index, function) in functions.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", function.r#type)?;
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
            (List::Boolean(a), List::Boolean(b)) => a == b,
            (List::Byte(a), List::Byte(b)) => a == b,
            (List::Character(a), List::Character(b)) => a == b,
            (List::Float(a), List::Float(b)) => a == b,
            (List::Integer(a), List::Integer(b)) => a == b,
            (List::String(a), List::String(b)) => a == b,
            (List::List(a), List::List(b)) => a == b,
            (List::Function(a), List::Function(b)) => a == b,
            _ => self.r#type() == other.r#type(),
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
            (List::Boolean(a), List::Boolean(b)) => a.cmp(b),
            (List::Byte(a), List::Byte(b)) => a.cmp(b),
            (List::Character(a), List::Character(b)) => a.cmp(b),
            (List::Float(a), List::Float(b)) => a
                .iter()
                .zip(b.iter())
                .find_map(|(a, b)| match a.to_bits().cmp(&b.to_bits()) {
                    Ordering::Equal => None,
                    other => Some(other),
                })
                .unwrap_or(Ordering::Equal),
            (List::Integer(a), List::Integer(b)) => a.cmp(b),
            (List::String(a), List::String(b)) => a.cmp(b),
            (List::List(a), List::List(b)) => a.cmp(b),
            (List::Function(a), List::Function(b)) => a.cmp(b),
            _ => self.r#type().cmp(&other.r#type()),
        }
    }
}

impl Hash for List {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            List::Boolean(booleans) => booleans.hash(state),
            List::Byte(bytes) => bytes.hash(state),
            List::Character(characters) => characters.hash(state),
            List::Float(floats) => {
                for float in floats {
                    float.to_bits().hash(state);
                }
            }
            List::Integer(items) => items.hash(state),
            List::String(strings) => strings.hash(state),
            List::List(lists) => lists.hash(state),
            List::Function(functions) => functions.hash(state),
        }
    }
}
