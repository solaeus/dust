use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{DustString, Type, chunk::Chunk};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum List<C> {
    Boolean(Vec<bool>),
    Byte(Vec<u8>),
    Character(Vec<char>),
    Float(Vec<f64>),
    Integer(Vec<i64>),
    String(Vec<DustString>),
    Function(Vec<Arc<C>>),
    List(Vec<List<C>>),
}

impl<C: Chunk> List<C> {
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

    pub fn list<T: Into<Vec<List<C>>>>(lists: T) -> Self {
        List::List(lists.into())
    }

    pub fn function<T: Into<Vec<Arc<C>>>>(functions: T) -> Self {
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
                .map(|function| Type::Function(Box::new(function.r#type().clone())))
                .unwrap_or(Type::None),
        }
    }

    pub fn r#type(&self) -> Type {
        Type::List(Box::new(self.item_type()))
    }
}

impl<C: Chunk> Display for List<C> {
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

                    write!(f, "{}", function.r#type())?;
                }
            }
        }

        write!(f, "]")
    }
}
