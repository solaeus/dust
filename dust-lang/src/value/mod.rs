mod list;

use std::fmt::{self, Display, Formatter};

pub use list::List;

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Character(char),
    String(String),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
    F32(f32),
    F64(f64),
    List(List),
    Function(u16),
    Struct {
        name: String,
        fields: Vec<(String, Value)>,
    },
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Character(character) => write!(f, "{character}"),
            Value::String(string) => write!(f, "{string}"),
            Value::U8(u8) => write!(f, "{u8}"),
            Value::I8(i8) => write!(f, "{i8}"),
            Value::U16(u16) => write!(f, "{u16}"),
            Value::I16(i16) => write!(f, "{i16}"),
            Value::U32(u32) => write!(f, "{u32}"),
            Value::I32(i32) => write!(f, "{i32}"),
            Value::U64(u64) => write!(f, "{u64}"),
            Value::I64(i64) => write!(f, "{i64}"),
            Value::U128(u128) => write!(f, "{u128}"),
            Value::I128(i128) => write!(f, "{i128}"),
            Value::F32(f32) => write!(f, "{f32}"),
            Value::F64(f64) => write!(f, "{f64}"),
            Value::List(list) => write!(f, "{list}"),
            Value::Function(prototype_index) => write!(f, "{prototype_index}"),
            Value::Struct { name, fields } => {
                write!(f, "{name} {{ ")?;

                for (i, (field_name, field_value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{field_name}: ")?;

                    if matches!(field_value, Value::String(_)) {
                        write!(f, "\"{field_value}\"")?;
                    } else {
                        write!(f, "{field_value}")?;
                    }
                }

                write!(f, " }}")
            }
        }
    }
}
