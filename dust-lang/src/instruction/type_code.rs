use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeCode(pub u8);

impl TypeCode {
    pub const NONE: TypeCode = TypeCode(0);
    pub const BOOLEAN: TypeCode = TypeCode(1);
    pub const BYTE: TypeCode = TypeCode(2);
    pub const CHARACTER: TypeCode = TypeCode(3);
    pub const FLOAT: TypeCode = TypeCode(4);
    pub const INTEGER: TypeCode = TypeCode(5);
    pub const STRING: TypeCode = TypeCode(6);
    pub const LIST: TypeCode = TypeCode(7);
    pub const FUNCTION: TypeCode = TypeCode(8);

    pub fn unknown_write(self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Malformed instruction: type code {} is unknown.", self.0)
    }

    pub fn unsupported_write(self, f: &mut Formatter) -> fmt::Result {
        write!("Malformed instruction: type code {self} is not supported here.")
    }
}

impl Display for TypeCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypeCode::NONE => fmt::Result::Ok(()),
            TypeCode::BOOLEAN => write!(f, "bool"),
            TypeCode::BYTE => write!(f, "byte"),
            TypeCode::CHARACTER => write!(f, "char"),
            TypeCode::FLOAT => write!(f, "float"),
            TypeCode::INTEGER => write!(f, "int"),
            TypeCode::STRING => write!(f, "str"),
            TypeCode::LIST => write!(f, "list"),
            TypeCode::FUNCTION => write!(f, "fn"),
            _ => self.unknown_write(f),
        }
    }
}
