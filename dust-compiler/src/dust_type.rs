use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DustType {
    /// Convenience variant for the unit type, which is the same as an empty tuple.
    Unit,

    /// `bool`: `true` or `false`
    Boolean,

    /// `i8`: -128..=127
    I8,

    /// `i16`: -32,768..=32,767
    I16,

    /// `i32`: -2,147,483,648..=2,147,483,647
    I32,

    /// `i64`: -2^63..=2^63-1
    I64,

    /// `i128`: -2^127..=2^127-1
    I128,

    /// `isize`: `i32` on 32-bit platforms, `i64` on 64-bit platforms
    ISize,

    /// `u8`: 0..=255
    U8,

    /// `u16`: 0..=65,535
    U16,

    /// `u32`: 0..=4,294,967,295
    U32,

    /// `u64`: 0..=2^64-1
    U64,

    /// `u128`: 0..=2^128-1
    U128,

    /// `usize`: `u32` on 32-bit platforms, `u64` on 64-bit platforms
    USize,

    /// `f32`: 32-bit floating-point number
    F32,

    /// `f64`: 64-bit floating-point number
    F64,

    /// `char`: a Unicode scalar value
    Character,

    /// `()`, `(T1, T2, ...)`
    Tuple(Vec<DustType>),

    /// `[T; N]`
    Array(Box<DustType>, usize),

    /// `fn<T1, T2, ...>(P1, P2, ...) -> R`
    Function(Box<DustFunctionType>),

    /// `struct Name { ... }`
    Struct(Box<DustStructType>),

    /// `enum Name { Variant1, Variant2, ... }`
    Enum(Box<DustEnumType>),

    /// `&T`
    Reference(Box<DustType>),

    /// A heap object pointer. Only used for internal purposes.
    Pointer,
}

impl Display for DustType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustType::Unit => write!(f, "()"),
            DustType::Boolean => write!(f, "bool"),
            DustType::Character => write!(f, "char"),
            DustType::I8 => write!(f, "i8"),
            DustType::I16 => write!(f, "i16"),
            DustType::I32 => write!(f, "i32"),
            DustType::I64 => write!(f, "i64"),
            DustType::I128 => write!(f, "i128"),
            DustType::ISize => write!(f, "isize"),
            DustType::U8 => write!(f, "u8"),
            DustType::U16 => write!(f, "u16"),
            DustType::U32 => write!(f, "u32"),
            DustType::U64 => write!(f, "u64"),
            DustType::U128 => write!(f, "u128"),
            DustType::USize => write!(f, "usize"),
            DustType::F32 => write!(f, "f32"),
            DustType::F64 => write!(f, "f64"),
            DustType::Tuple(item_type) => {
                write!(f, "(")?;

                for (index, r#type) in item_type.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{type}")?;
                }

                write!(f, ")")
            }
            DustType::Array(item_type, size) => write!(f, "[{item_type}; {size}]"),
            DustType::Function(function_type) => write!(f, "{function_type}"),
            DustType::Struct(struct_type) => write!(f, "{struct_type}"),
            DustType::Enum(enum_type) => write!(f, "{enum_type}"),
            DustType::Reference(referenced_type) => write!(f, "&{referenced_type}"),
            DustType::Pointer => write!(f, "*pointer"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DustFunctionType {
    pub value_parameters: Vec<DustType>,
    pub return_type: DustType,
}

impl DustFunctionType {
    pub fn new<T: Into<Vec<DustType>>>(value_parameters: T, return_type: DustType) -> Self {
        DustFunctionType {
            value_parameters: value_parameters.into(),
            return_type,
        }
    }
}

impl Display for DustFunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn (")?;

        if !self.value_parameters.is_empty() {
            for (index, r#type) in self.value_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{type}")?;
            }
        }

        write!(f, ") -> {}", self.return_type)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DustStructType {
    pub name: String,
    pub value_type: DustStructTypeFields,
}

impl Display for DustStructType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "struct {}{}", self.name, self.value_type)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DustEnumType {
    pub name: String,
    pub variants: Vec<(String, DustStructTypeFields)>,
}

impl Display for DustEnumType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "enum {} {{", self.name)?;

        for (index, (variant_name, variant_value_type)) in self.variants.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{variant_name}{variant_value_type}")?;
        }

        write!(f, "}}")
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DustStructTypeFields {
    Unit,
    Tuple(Vec<DustType>),
    Named(Vec<(String, DustType)>),
}

impl Display for DustStructTypeFields {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustStructTypeFields::Unit => Ok(()),
            DustStructTypeFields::Tuple(types) => {
                write!(f, "(")?;

                for (index, r#type) in types.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{type}")?;
                }

                write!(f, ")")
            }
            DustStructTypeFields::Named(fields) => {
                if fields.len() == 1 {
                    let (field_name, field_type) = &fields[0];

                    return write!(f, "{{ {field_name}: {field_type} }}");
                }

                writeln!(f, "{{")?;

                for (field_name, field_type) in fields {
                    writeln!(f, "{field_name}: {field_type},")?;
                }

                writeln!(f, "}}")
            }
        }
    }
}
