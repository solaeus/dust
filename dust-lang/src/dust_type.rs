use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DustType {
    #[default]
    Unit,
    Boolean,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    F32,
    F64,
    Character,
    Tuple(Box<DustType>),
    Array(Box<DustType>, usize),
    Slice(Box<DustType>),
    Function(Box<DustFunctionType>),
    Struct(Box<DustStructType>),
    Enum(String, Vec<DustStructType>),
}

impl DustType {
    pub fn into_function_type(self) -> Option<DustFunctionType> {
        match self {
            DustType::Function(function_type) => Some(*function_type),
            _ => None,
        }
    }
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
            DustType::Tuple(item_type) => write!(f, "({item_type})"),
            DustType::Array(item_type, size) => write!(f, "[{item_type}; {size}]"),
            DustType::Slice(item_type) => write!(f, "[{item_type}]"),
            DustType::Function(function_type) => write!(f, "{function_type}"),
            DustType::Struct(struct_type) => write!(f, "{struct_type}"),
            DustType::Enum(name, variants) => {
                write!(f, "enum {name} {{")?;

                for (index, variant) in variants.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{variant}")?;
                }

                write!(f, "}}")
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DustFunctionType {
    pub type_parameters: Vec<String>,
    pub value_parameters: Vec<DustType>,
    pub return_type: DustType,
}

impl DustFunctionType {
    pub fn new<T: Into<Vec<String>>, U: Into<Vec<DustType>>>(
        type_parameters: T,
        value_parameters: U,
        return_type: DustType,
    ) -> Self {
        DustFunctionType {
            type_parameters: type_parameters.into(),
            value_parameters: value_parameters.into(),
            return_type,
        }
    }
}

impl Display for DustFunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn")?;

        if !self.type_parameters.is_empty() {
            write!(f, "<")?;

            for (index, type_parameter_name) in self.type_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{type_parameter_name}")?;
            }

            write!(f, ">")?;
        }

        write!(f, "(")?;

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

#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DustStructType {
    pub name: String,
    pub fields: Vec<(String, DustType)>,
}

impl Display for DustStructType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "struct {} {{", self.name)?;

        for (index, (field_name, field_type)) in self.fields.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{field_name}: {field_type}")?;
        }

        write!(f, "}}")
    }
}
