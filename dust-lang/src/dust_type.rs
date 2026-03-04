use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DustType {
    #[default]
    Unit,
    Boolean,
    Character,
    String,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    F32,
    F64,
    List(Box<DustType>),
    Function(Box<DustFunctionType>),
    Struct(Box<DustStructType>),
}

impl DustType {
    pub fn list(element_type: DustType) -> Self {
        DustType::List(Box::new(element_type))
    }

    pub fn function<T: Into<Vec<String>>, U: Into<Vec<DustType>>>(
        type_parameters: T,
        value_parameters: U,
        return_type: DustType,
    ) -> Self {
        DustType::Function(Box::new(DustFunctionType {
            type_parameters: type_parameters.into(),
            value_parameters: value_parameters.into(),
            return_type,
        }))
    }

    pub fn as_element_type(&self) -> Option<&DustType> {
        match self {
            DustType::List(item_type) => Some(item_type.as_ref()),
            _ => None,
        }
    }

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
            DustType::Unit => write!(f, "none"),
            DustType::Boolean => write!(f, "bool"),
            DustType::Character => write!(f, "char"),
            DustType::String => write!(f, "str"),
            DustType::U8 => write!(f, "u8"),
            DustType::I8 => write!(f, "i8"),
            DustType::U16 => write!(f, "u16"),
            DustType::I16 => write!(f, "i16"),
            DustType::U32 => write!(f, "u32"),
            DustType::I32 => write!(f, "i32"),
            DustType::U64 => write!(f, "u64"),
            DustType::I64 => write!(f, "i64"),
            DustType::U128 => write!(f, "u128"),
            DustType::I128 => write!(f, "i128"),
            DustType::F32 => write!(f, "f32"),
            DustType::F64 => write!(f, "f64"),
            DustType::Function(function_type) => write!(f, "{function_type}"),
            DustType::List(item_type) => write!(f, "[{item_type}]"),
            DustType::Struct(struct_type) => write!(f, "{struct_type}"),
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
        write!(f, "{}", self.name)
    }
}
