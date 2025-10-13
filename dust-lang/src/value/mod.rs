mod list;

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

pub use list::List;

use crate::instruction::OperandType;

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String(String),
    Array(Vec<Value>),
    List(List),
    Function(u16),
}

impl Value {
    pub fn boolean(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Boolean(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn byte(byte: u8) -> Self {
        Value::Byte(byte)
    }

    pub fn as_byte(&self) -> Option<u8> {
        if let Value::Byte(byte) = self {
            Some(*byte)
        } else {
            None
        }
    }

    pub fn character(character: char) -> Self {
        Value::Character(character)
    }

    pub fn as_character(&self) -> Option<char> {
        if let Value::Character(character) = self {
            Some(*character)
        } else {
            None
        }
    }

    pub fn float(float: f64) -> Self {
        Value::Float(float)
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(float) = self {
            Some(*float)
        } else {
            None
        }
    }

    pub fn integer(integer: i64) -> Self {
        Value::Integer(integer)
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let Value::Integer(integer) = self {
            Some(*integer)
        } else {
            None
        }
    }

    pub fn string<T: Into<String>>(value: T) -> Self {
        Value::String(value.into())
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn boolean_list<T: Into<Vec<bool>>>(booleans: T) -> Self {
        Value::List(List::boolean(booleans))
    }

    pub fn byte_list<T: Into<Vec<u8>>>(bytes: T) -> Self {
        Value::List(List::byte(bytes))
    }

    pub fn character_list<T: Into<Vec<char>>>(characters: T) -> Self {
        Value::List(List::character(characters))
    }

    pub fn float_list<T: Into<Vec<f64>>>(floats: T) -> Self {
        Value::List(List::float(floats))
    }

    pub fn integer_list<T: Into<Vec<i64>>>(items: T) -> Self {
        Value::List(List::integer(items))
    }

    pub fn string_list<T: Into<Vec<String>>>(strings: T) -> Self {
        Value::List(List::string(strings))
    }

    pub fn list_list<T: Into<Vec<List>>>(lists: T) -> Self {
        Value::List(List::list(lists))
    }

    pub fn function_list<T: Into<Vec<usize>>>(prototype_indexes: T) -> Self {
        Value::List(List::function(prototype_indexes))
    }

    pub fn as_list(&self) -> Option<&List> {
        if let Value::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn into_list(self) -> Option<List> {
        if let Value::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn function(prototype_index: u16) -> Self {
        Value::Function(prototype_index)
    }

    pub fn as_function(&self) -> Option<u16> {
        if let Value::Function(prototype_index) = self {
            Some(*prototype_index)
        } else {
            None
        }
    }

    pub fn operand_type(&self) -> OperandType {
        match self {
            Value::Boolean(_) => OperandType::BOOLEAN,
            Value::Byte(_) => OperandType::BYTE,
            Value::Character(_) => OperandType::CHARACTER,
            Value::Float(_) => OperandType::FLOAT,
            Value::Integer(_) => OperandType::INTEGER,
            Value::String(_) => OperandType::STRING,
            Value::Array(values) => {
                let value_type = if let Some(first) = values.first() {
                    first.operand_type()
                } else {
                    OperandType::NONE
                };

                match value_type {
                    OperandType::BOOLEAN => OperandType::ARRAY_BOOLEAN,
                    OperandType::BYTE => OperandType::ARRAY_BYTE,
                    OperandType::CHARACTER => OperandType::ARRAY_CHARACTER,
                    OperandType::FLOAT => OperandType::ARRAY_FLOAT,
                    OperandType::INTEGER => OperandType::ARRAY_INTEGER,
                    OperandType::STRING => OperandType::ARRAY_STRING,
                    OperandType::FUNCTION => OperandType::ARRAY_FUNCTION,
                    _ => todo!(),
                }
            }
            Value::List(list) => list.operand_type(),
            Value::Function(_) => OperandType::FUNCTION,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Byte(byte) => write!(f, "{byte}"),
            Value::Character(character) => write!(f, "{character}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::String(string) => write!(f, "{string}"),
            Value::Array(array) => {
                write!(f, "[")?;

                for (i, value) in array.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{value}")?;
                }

                write!(f, "]")
            }
            Value::List(list) => write!(f, "{list}"),
            Value::Function(chunk) => write!(f, "{chunk}"),
        }
    }
}

impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Byte(left), Value::Byte(right)) => left == right,
            (Value::Character(left), Value::Character(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left.to_bits() == right.to_bits(),
            (Value::Integer(left), Value::Integer(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::List(left), Value::List(right)) => left == right,
            (Value::Function(left), Value::Function(right)) => left == right,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => left.cmp(right),
            (Value::Boolean(_), _) => Ordering::Less,
            (Value::Byte(left), Value::Byte(right)) => left.cmp(right),
            (Value::Byte(_), _) => Ordering::Less,
            (Value::Character(left), Value::Character(right)) => left.cmp(right),
            (Value::Character(_), _) => Ordering::Less,
            (Value::Float(left), Value::Float(right)) => left.total_cmp(right),
            (Value::Float(_), _) => Ordering::Less,
            (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
            (Value::Integer(_), _) => Ordering::Less,
            (Value::String(left), Value::String(right)) => left.cmp(right),
            (Value::String(_), _) => Ordering::Less,
            (Value::Array(left), Value::Array(right)) => left.cmp(right),
            (Value::Array(_), _) => Ordering::Less,
            (Value::List(left), Value::List(right)) => left.cmp(right),
            (Value::List(_), _) => Ordering::Less,
            (Value::Function(left), Value::Function(right)) => left.cmp(right),
            (Value::Function(_), _) => Ordering::Greater,
        }
    }
}

// impl Hash for Value {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         match self {
//             Value::Boolean(value) => value.hash(state),
//             Value::Byte(value) => value.hash(state),
//             Value::Character(value) => value.hash(state),
//             Value::Float(value) => value.to_bits().hash(state),
//             Value::Integer(value) => value.hash(state),
//             Value::String(value) => value.hash(state),
//             Value::List(value) => value.hash(state),
//             Value::Function(value) => value.hash(state),
//         }
//     }
// }
