use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TypeCode(pub u8);

impl TypeCode {
    pub const BOOLEAN: TypeCode = TypeCode(0);
    pub const BYTE: TypeCode = TypeCode(1);
    pub const CHARACTER: TypeCode = TypeCode(2);
    pub const FLOAT: TypeCode = TypeCode(3);
    pub const INTEGER: TypeCode = TypeCode(4);
    pub const STRING: TypeCode = TypeCode(5);

    pub fn panic_from_unknown_code(self) -> ! {
        panic!("Unknown type code: {}", self.0);
    }

    pub fn register_name(self) -> &'static str {
        match self {
            TypeCode::BOOLEAN => "R_BOOL",
            TypeCode::BYTE => "R_BYTE",
            TypeCode::CHARACTER => "R_CHAR",
            TypeCode::FLOAT => "R_FLOAT",
            TypeCode::INTEGER => "R_INT",
            TypeCode::STRING => "R_STR",
            _ => self.panic_from_unknown_code(),
        }
    }
}

impl Display for TypeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TypeCode::BOOLEAN => write!(f, "bool"),
            TypeCode::BYTE => write!(f, "byte"),
            TypeCode::CHARACTER => write!(f, "char"),
            TypeCode::FLOAT => write!(f, "float"),
            TypeCode::INTEGER => write!(f, "int"),
            TypeCode::STRING => write!(f, "str"),
            _ => self.panic_from_unknown_code(),
        }
    }
}
