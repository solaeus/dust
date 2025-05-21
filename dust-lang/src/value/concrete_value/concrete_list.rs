use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{Chunk, DustString, FunctionType, Type};

use super::ConcreteRange;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ConcreteList {
    Boolean(Vec<bool>),
    Byte(Vec<u8>),
    Character(Vec<char>),
    Float(Vec<f64>),
    Function {
        functions: Vec<Arc<Chunk>>,
        function_type: Box<FunctionType>,
    },
    Integer(Vec<i64>),
    List {
        list_items: Vec<ConcreteList>,
        list_item_type: Type,
    },
    Range {
        ranges: Vec<ConcreteRange>,
        range_type: Type,
    },
    String(Vec<DustString>),
}

impl ConcreteList {
    pub fn of_lists(list_items: Vec<ConcreteList>) -> Self {
        let list_item_type = list_items
            .first()
            .map(|list| list.r#type())
            .unwrap_or(Type::None);

        ConcreteList::List {
            list_items,
            list_item_type,
        }
    }

    pub fn item_type(&self) -> Type {
        match self {
            ConcreteList::Boolean(_) => Type::Boolean,
            ConcreteList::Byte(_) => Type::Byte,
            ConcreteList::Character(_) => Type::Character,
            ConcreteList::Float(_) => Type::Float,
            ConcreteList::Function { function_type, .. } => Type::Function(function_type.clone()),
            ConcreteList::Integer(_) => Type::Integer,
            ConcreteList::List { list_item_type, .. } => list_item_type.clone(),
            ConcreteList::Range { range_type, .. } => range_type.clone(),
            ConcreteList::String(_) => Type::String,
        }
    }

    pub fn r#type(&self) -> Type {
        Type::List(Box::new(self.item_type()))
    }
}

impl Display for ConcreteList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;

        match self {
            ConcreteList::Boolean(booleans) => {
                for (index, boolean) in booleans.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{boolean}")?;
                }
            }
            ConcreteList::Byte(bytes) => {
                for (index, byte) in bytes.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{byte}")?;
                }
            }
            ConcreteList::Character(characters) => {
                for (index, character) in characters.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{character}")?;
                }
            }
            ConcreteList::Float(floats) => {
                for (index, float) in floats.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{float}")?;
                }
            }
            ConcreteList::Function { functions, .. } => {
                for (index, function) in functions.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{function}")?;
                }
            }
            ConcreteList::Integer(items) => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{item}")?;
                }
            }
            ConcreteList::List { list_items, .. } => {
                for (index, list) in list_items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{list}")?;
                }
            }
            ConcreteList::Range { ranges, .. } => {
                for (index, range) in ranges.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{range}")?;
                }
            }
            ConcreteList::String(strings) => {
                for (index, string) in strings.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{string}")?;
                }
            }
        }

        write!(f, "]")
    }
}
