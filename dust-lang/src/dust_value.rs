use std::fmt::{self, Display, Formatter};

/// External representation of a Dust value.
///
/// This is used to represent values going into and out of the VM. It is not used as a
/// representation of values in the compiler or the VM.
pub enum DustValue {
    Boolean(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Character(char),
}

impl Display for DustValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustValue::Boolean(boolean) => write!(f, "{boolean}"),
            DustValue::I8(integer) => write!(f, "{integer}"),
            DustValue::I16(integer) => write!(f, "{integer}"),
            DustValue::I32(integer) => write!(f, "{integer}"),
            DustValue::I64(integer) => write!(f, "{integer}"),
            DustValue::I128(integer) => write!(f, "{integer}"),
            DustValue::U8(integer) => write!(f, "{integer}"),
            DustValue::U16(integer) => write!(f, "{integer}"),
            DustValue::U32(integer) => write!(f, "{integer}"),
            DustValue::U64(integer) => write!(f, "{integer}"),
            DustValue::U128(integer) => write!(f, "{integer}"),
            DustValue::F32(float) => write!(f, "{float}"),
            DustValue::F64(float) => write!(f, "{float}"),
            DustValue::Character(character) => write!(f, "'{character}'"),
        }
    }
}
