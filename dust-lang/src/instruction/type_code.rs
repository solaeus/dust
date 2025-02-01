use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeCode(pub u8);

impl TypeCode {
    pub const NONE: TypeCode = TypeCode(0);
    pub const BOOLEAN: TypeCode = TypeCode(1);
    pub const BYTE: TypeCode = TypeCode(2);
    pub const CHARACTER: TypeCode = TypeCode(3);
    pub const FLOAT: TypeCode = TypeCode(4);
    pub const INTEGER: TypeCode = TypeCode(5);
    pub const STRING: TypeCode = TypeCode(6);

    pub fn panic_from_unknown_code(self) -> ! {
        panic!("Unknown type code: {}", self.0);
    }

    pub fn panic_from_unsupported_code(self) -> ! {
        panic!("Type code {} is not supported in the context", self.0,);
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
